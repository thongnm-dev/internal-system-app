export type UserProfile = {
  username: string;
  password: string;
  full_name: string;
  email: string;
  phone: string;
  address: string;
  position: string;
};

export type AppSettings = {
  user: UserProfile;
  theme: string;
  language: string;
};

export type SaveSettingsRequest = {
  user_id: number;
  user: UserProfile;
  theme: string;
  language: string;
};
