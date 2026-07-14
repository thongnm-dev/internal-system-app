export type LoginRequest = {
  username: string;
  password: string;
};

export type LoginResponse = {
  user_id: number;
  username: string;
  full_name: string;
  email: string;
  roles: string[];
};
