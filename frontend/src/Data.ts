export interface Person {
  id: string;
}

export interface Pub {
  id: string;
  name: string;
  persons: Person[];
}

export interface Table {
  id: string;
  name: string;
  persons: Person[];
}
