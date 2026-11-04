use std::sync::Arc;

use crate::domain::error::PowerError;
use crate::domain::ports::PowerControl;
use crate::domain::power::PowerAction;

/// Menjalankan perintah daya. Konfirmasi "yakin?" ada di kartu pulau; layanan ini hanya meneruskan.
pub struct PowerService {
    control: Arc<dyn PowerControl>,
}

impl PowerService {
    pub fn new(control: Arc<dyn PowerControl>) -> Arc<Self> {
        Arc::new(Self { control })
    }

    pub fn perform(&self, action: PowerAction) -> Result<(), PowerError> {
        tracing::info!("perintah daya: {action:?}");
        self.control.perform(action)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct Fake(Mutex<Vec<PowerAction>>);

    impl PowerControl for Fake {
        fn perform(&self, action: PowerAction) -> Result<(), PowerError> {
            self.0.lock().unwrap().push(action);
            Ok(())
        }
    }

    #[test]
    fn aksi_diteruskan_ke_port() {
        let fake = Arc::new(Fake::default());
        let svc = PowerService::new(fake.clone());
        svc.perform(PowerAction::Sleep).unwrap();
        svc.perform(PowerAction::Lock).unwrap();
        assert_eq!(
            *fake.0.lock().unwrap(),
            [PowerAction::Sleep, PowerAction::Lock]
        );
    }
}
