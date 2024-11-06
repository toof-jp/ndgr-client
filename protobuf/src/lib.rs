pub mod chat {
    pub mod data {
        pub mod atoms {
            include!(concat!(
                env!("OUT_DIR"),
                "/dwango.nicolive.chat.data.atoms.rs"
            ));
        }
        include!(concat!(env!("OUT_DIR"), "/dwango.nicolive.chat.data.rs"));
    }
    pub mod service {
        pub mod edge {
            include!(concat!(
                env!("OUT_DIR"),
                "/dwango.nicolive.chat.service.edge.rs"
            ));
        }
    }
}
