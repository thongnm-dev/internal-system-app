export interface MenuConfig {
  key: string;
  title: string;
  path: string;
  icon: string;
  group: string;
  visible: boolean;
  order: number;
}

export interface SaveMenuConfigRequest {
  key: string;
  title: string;
  path: string;
  icon: string;
  group: string;
  visible: boolean;
  order: number;
}

export interface SaveAllMenuConfigsRequest {
  items: SaveMenuConfigRequest[];
}
