use clap::arg_enum;

arg_enum! {
    #[derive(Debug, PartialEq)]
    pub enum EventType {
        Ack,
        Data,

        BootloaderHello,

        ProgrammerHello,
        ProgrammerStartUpload,

        BcmChangeBrightness,
    }
}
