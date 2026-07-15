export type UserProfile = {
  username: string;
  password: string;
  full_name: string;
  email: string;
  phone: string;
  address: string;
  position: string;
};

export type ApiKeySetting = {
  id: string;
  name: string;
  key_label: string;
  api_key: string;
};

export type AppSettings = {
  user: UserProfile;
  theme: string;
  language: string;
  api_keys: ApiKeySetting[];
};

export type SaveSettingsRequest = {
  user_id: number;
  user: UserProfile;
  theme: string;
  language: string;
  api_keys: ApiKeySetting[];
};
