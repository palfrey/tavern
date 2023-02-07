export interface Person {
  id: string;
  pub_id: string | null;
  table_id: string | null;
  name: string | null;
  last_updated: string;
}

export interface Pub {
  id: string;
  name: string;
  persons: string[];
}

export interface Table {
  id: string;
  name: string;
  persons: string[];
}
