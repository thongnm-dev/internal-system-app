export type RoleSummary = {
  id: number;
  name: string;
  description: string;
  user_count: number;
  created_at: string;
};

export type CreateRoleRequest = {
  name: string;
  description?: string;
};

export type UpdateRoleRequest = {
  name: string;
  description?: string;
};
