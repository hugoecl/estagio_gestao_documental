import type { MenuItem } from "@lib/types/menu";

export const menuItems: MenuItem[] = [
  {
    title: "Contratos",
    path: "/contratos/",
  },
  {
    title: "Qualidade (SGQ)",
    parentPath: "/qualidade-sgq",
    children: [
      { title: "Certificado", path: "certificado/" },
      { title: "Manual", path: "manual/" },
      {
        title: "PO",
        parentPath: "po",
        children: [
          { title: "P1", path: "p1/" },
          { title: "P2", path: "p2/" },
        ],
      },
      { title: "Módulos", path: "modulos/" },
      { title: "Não Conformidade", path: "nao-conformidade/" },
      {
        title: "Fornecedores",
        parentPath: "fornecedores",
        children: [
          {
            title: "Fornecedores Qualificados",
            path: "fornecedores-qualificados/",
          },
          {
            title: "Desempenho do Fornecedor",
            path: "desempenho-do-fornecedor/",
          },
        ],
      },
      { title: "Objetivos de Qualidade", path: "objetivos-qualidade/" },
      {
        title: "Inquérito de Satisfação",
        parentPath: "inquerito-satisfacao",
        children: [
          { title: "Clientes Particulares", path: "clientes-particulares/" },
          { title: "Outros", path: "outros/" },
        ],
      },
      {
        title: "Revisão pela Gestão",
        parentPath: "revisao-gestao",
        children: [
          { title: "Relatórios", path: "relatorios/" },
          { title: "Atas", path: "atas/" },
          { title: "Análise SWOT", path: "analise swot/" },
          {
            title: "Necessidades e Expectativas das Partes Interessadas",
            parentPath: "necessidades-expectativas-partes-interessadas",
            children: [
              { title: "Avaliação", path: "avaliacao/" },
              { title: "Plano", path: "plano/" },
            ],
          },
        ],
      },
      {
        title: "Auditorias",
        parentPath: "auditorias",
        children: [
          { title: "Internas", path: "internas/" },
          { title: "Externas", path: "externas/" },
          {
            title: "Entidade Certificadora",
            children: [
              { title: "Resposta Anterior", path: "resposta-anterior/" },
            ],
          },
        ],
      },
    ],
  },
  {
    title: "Recursos Humanos",
    parentPath: "/recursos-humanos",
    children: [
      { title: "Categorias Profissionais", path: "categorias-profissionais/" },
      { title: "Contratos de Funcionários", path: "contratos-funcionarios/" },
    ],
  },
  {
    title: "ERS",
    parentPath: "/ers",
    children: [
      { title: "Geral", path: "geral/" },
      {
        title: "Proteção Radiológica",
        parentPath: "protecao-radiologica",
        children: [{ title: "Licenças", path: "licencas/" }],
      },
      {
        title: "Inspeção",
        path: "inspecao/",
      },
    ],
  },
  {
    title: "APA",
    path: "/apa/",
  },
];

export function extractPathsAndTitles(
  items: MenuItem[],
  basePath: string = ""
): Record<string, string> {
  let result: Record<string, string> = {};

  for (const item of items) {
    if (item.path) {
      // Handle direct path (full or relative)
      const fullPath = item.path.startsWith("/")
        ? item.path
        : basePath + item.path;
      result[fullPath] = item.title;

      if (fullPath !== "/") {
        result[fullPath] = item.title;
      }
    }

    if (item.children) {
      // Handle items with children
      const newBasePath = item.parentPath
        ? item.parentPath.startsWith("/")
          ? item.parentPath + "/"
          : basePath + item.parentPath + "/"
        : basePath;

      const childPaths = extractPathsAndTitles(item.children, newBasePath);
      result = { ...result, ...childPaths };
    }
  }

  return result;
}
