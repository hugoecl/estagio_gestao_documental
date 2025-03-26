export const pageIcons: Record<string, string> = {
  "/contratos/": "file-contract",
  "/qualidade-sgq/certificado/": "certificate",
  "/qualidade-sgq/manual/": "book",
  "/qualidade-sgq/po/p1/": "file-lines",
  "/qualidade-sgq/po/p2/": "file-lines",
  "/qualidade-sgq/modulos/": "cubes",
  "/qualidade-sgq/nao-conformidade/": "triangle-exclamation",
  "/qualidade-sgq/fornecedores/fornecedores-qualificados/": "users",
  "/qualidade-sgq/fornecedores/desempenho-do-fornecedor/": "chart-line",
  "/recursos-humanos/categorias-profissionais/": "file-contract",
  "/ers/protecao-radiologica/": "shield",
};
export function getIcon(path: string): string {
  return pageIcons[path] || "link";
}

export function getName(pages: Record<string, string>, path: string): string {
  return pages[path] || path.replace(/\/$/, "").split("/").pop() || path;
}
