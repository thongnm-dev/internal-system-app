export type UserDetail = {
  id: number;
  username: string;
  full_name: string;
  email: string;
  phone: string;
  address: string;
  position: string;
  is_active: boolean;
  roles: string[];
  created_at: string;
  updated_at: string;
};

export type UserSummary = {
  id: number;
  username: string;
  full_name: string;
  email: string;
  phone: string;
  position: string;
  is_active: boolean;
  roles: string[];
  created_at: string;
  updated_at: string;
};

export type CreateUserRequest = {
  username: string;
  password: string;
  full_name: string;
  email?: string;
  phone?: string;
  address?: string;
  position?: string;
  roles: string[];
};

export type UpdateUserRequest = {
  full_name: string;
  email?: string;
  phone?: string;
  address?: string;
  position?: string;
  is_active: boolean;
  roles: string[];
};

export type ChangePasswordRequest = {
  new_password: string;
};
