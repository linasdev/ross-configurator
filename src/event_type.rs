use clap::arg_enum;

arg_enum! {
    #[derive(Debug, PartialEq)]
    pub enum EventType {
        Ack,
        Data,

        ConfiguratorHello,

        BootloaderHello,

        ProgrammerHello,
        ProgrammerStartFirmwareUpgrade,
        ProgrammerStartConfigUpgrade,

        ButtonPressed,
        ButtonReleased,

        SystemTick
    }
}
