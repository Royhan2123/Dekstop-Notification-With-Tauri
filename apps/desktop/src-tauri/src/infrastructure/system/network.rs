use crate::domain::ports::NetCounters;

/// Total byte semua adaptor jaringan fisik (Wi-Fi, Ethernet) sejak boot.
/// Adaptor virtual (VPN, Hyper-V, WSL) sengaja tidak dihitung supaya lalu lintas tidak terhitung dua kali.
pub struct SystemNet;

/// macOS: jumlahkan byte masuk/keluar adaptor `en*` (Wi-Fi, Ethernet, adaptor USB/Thunderbolt) dari
/// keluaran `netstat -ib`. Loopback (`lo0`), VPN (`utun*`), AirDrop (`awdl*`), dan bridge dilewati.
/// Kolom dibaca dari kanan karena kolom Address bisa kosong: ... Ipkts Ierrs Ibytes Opkts Oerrs
/// Obytes Coll. Hanya baris `<Link#N>` (total per adaptor); baris alamat IP mengulang angka yang sama.
#[cfg(any(target_os = "macos", test))]
fn parse_netstat(output: &str) -> (u64, u64) {
    let mut sum = (0u64, 0u64);
    for line in output.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.len() < 10 || !t[0].starts_with("en") || !t.iter().any(|x| x.starts_with("<Link#")) {
            continue;
        }
        let n = t.len();
        if let (Ok(ibytes), Ok(obytes)) = (t[n - 5].parse::<u64>(), t[n - 2].parse::<u64>()) {
            sum.0 += ibytes;
            sum.1 += obytes;
        }
    }
    sum
}

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
        #[cfg(target_os = "macos")]
        {
            let out = std::process::Command::new("netstat")
                .arg("-ib")
                .output()
                .ok()?;
            out.status
                .success()
                .then(|| parse_netstat(&String::from_utf8_lossy(&out.stdout)))
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Name       Mtu   Network       Address            Ipkts Ierrs     Ibytes    Opkts Oerrs     Obytes  Coll
lo0        16384 <Link#1>                          100     0       5000      100     0       5000     0
lo0        16384 127           127.0.0.1           100     -       5000      100     -       5000     -
en0        1500  <Link#6>    a4:83:e7:12:34:56   49302     0   31234567    30211     0    5432109     -
en0        1500  192.168.1.0/24 192.168.1.10       49302     -   31234567    30211     -    5432109     -
en1        1500  <Link#7>    f2:00:00:00:00:01       0     0          0        0     0          0     0
utun4      1380  <Link#12>                         2000     0     900000     1500     0     800000     0
awdl0      1500  <Link#10>   aa:bb:cc:dd:ee:ff       10     0       1000       20     0       2000     0
bridge0    1500  <Link#11>   f2:00:00:00:00:02       5     0        300        5     0        300     0
en5        1500  <Link#9>                           700     0     123456      600     0      65432     0
";

    #[test]
    fn netstat_menjumlah_hanya_adaptor_en_dan_baris_link() {
        // en0 + en1 + en5; lo0, utun (VPN), awdl, bridge, dan baris alamat IP tidak dihitung.
        assert_eq!(
            parse_netstat(SAMPLE),
            (31_234_567 + 123_456, 5_432_109 + 65_432)
        );
        assert_eq!(parse_netstat(""), (0, 0));
        assert_eq!(parse_netstat("rusak\nen0 1 <Link#1> x"), (0, 0));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn netstat_asli_di_mac_terbaca() {
        assert!(SystemNet.totals().is_some());
    }

    #[test]
    fn membaca_total_jaringan_tidak_crash() {
        // Pada CI tanpa adaptor fisik hasilnya nol atau None; yang penting tidak panik.
        let _ = SystemNet.totals();
    }
}
