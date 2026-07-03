mod app {
    pub mod consts;
    pub mod error;
    pub mod result;
}

mod commands {
    pub mod excel2md_commands;
    pub mod import_commands;
    pub mod project_commands;
    pub mod system_commands;
}

mod database {
    pub mod csv_reader;
    pub mod report_store;
}

mod models {
    pub mod excel2md;
    pub mod import_csv;
    pub mod monthly_report;
    pub mod project;
    pub mod system;
}

mod services {
    pub mod excel2md_service;
    pub mod import_csv_service;
    pub mod monthly_report_service;
    pub mod project_service;
    pub mod system_service;
}

mod utils {
    #[allow(dead_code)]
    pub mod api_client;
    pub mod csv_reader;
    pub mod network;
    pub mod time;
}
