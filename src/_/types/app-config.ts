export type ConfigEntry = {
  key: string;
  value: string;
};

export type ConfigSection = {
  name: string;
  entries: ConfigEntry[];
};

export type AppConfigData = {
  sections: ConfigSection[];
  config_path: string;
};

export type SaveAppConfigRequest = {
  sections: ConfigSection[];
};

export type SpInfo = {
  name: string;
  file_name: string;
};

export type SpExecutionResult = {
  name: string;
  success: boolean;
  message: string;
};

export type SpExecutionSummary = {
  total: number;
  success_count: number;
  error_count: number;
  results: SpExecutionResult[];
};
