use crate::domain::ports::NetCounters;

/// Total byte semua adaptor jaringan fisik (Wi-Fi, Ethernet) sejak boot.
/// Adaptor virtual (VPN, Hyper-V, WSL) sengaja tidak dihitung supaya lalu lintas tidak terhitung dua kali.
pub struct SystemNet;

impl NetCounters for SystemNet {
    fn totals(&self) -> Option<(u64, u64)> {
        #[cfg(windows)]
        {
            use windows::Win32::NetworkManagement::IpHelper::{
                FreeMibTable, GetIfTable2, MIB_IF_TABLE2,
            };
            use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;

            let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
            // SAFETY: GetIfTable2 mengisi `table` dengan blok milik OS; dibaca selama masih valid
            // lalu dibebaskan dengan FreeMibTable. Baris dibaca lewat panjang `NumEntries`.
            unsafe {
                GetIfTable2(&mut table).ok().ok()?;
                let t = &*table;
                let rows = std::slice::from_raw_parts(t.Table.as_ptr(), t.NumEntries as usize);
                let mut sum = (0u64, 0u64);
                for r in rows {
                    // Bit 0 InterfaceAndOperStatusFlags = HardwareInterface.
                    let hardware = r.InterfaceAndOperStatusFlags._bitfield & 1 != 0;
                    if hardware && r.OperStatus == IfOperStatusUp {
                        sum.0 += r.InOctets;
                        sum.1 += r.OutOctets;
                    }
                }
                FreeMibTable(table as *const _);
                Some(sum)
            }
        }
        #[cfg(not(windows))]
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membaca_total_jaringan_tidak_crash() {
        // Pada CI tanpa adaptor fisik hasilnya nol atau None; yang penting tidak panik.
        let _ = SystemNet.totals();
    }
}
