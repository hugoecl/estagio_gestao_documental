export type MenuItem =
  | {
      title: string;
      path: string;
      children?: never; // No children allowed when path is present
    }
  | {
      title: string;
      path?: never; // No path allowed when children are present
      parentPath?: string;
      children: MenuItem[];
    };
