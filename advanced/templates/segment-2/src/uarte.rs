use dwm1001::nrf52832_hal::nrf52832_pac::UARTE0;

pub struct Uarte {
    periph: UARTE0,
}

impl Uarte {
    pub fn new(periph: UARTE0) -> Self {
        Self {
            periph
        }
    }
}
