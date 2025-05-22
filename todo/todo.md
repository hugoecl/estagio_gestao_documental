# TODO

- [X] Administrador poder criar perfis
- [X] Adicionar notificações para validades ou seja cada campo de validade vai passar a ter mais um opção para notificação (se é para notificar ou nao e com quanto tempo de antecedencia)
- [X] Adicionar pagina de "Comunicações/Documentos Importantes". O admin vai adicionando documentos em que os colaborador tem de "assinar" que tomaram conhecimento. Quando o admin adicionar tal documento, os colaboradores serão notificados. Quando um colaborador clica para abrir um documento novo, aparece um popup a dizer que concorda que ao abrir este documento esta a tomar conhecimento do mesmo. O admin depois consegue ver com tomou conhecimento
- [X] Simplificar a criação de paginas. Fazer com que o caminho seja automatico e selecionar grupo
- [X] Traduzir campos, formatar datas em funções, e traduzir true/false em funcoes
- [X] Meter verificações de campos apenas em TEXT
- [X] Quando tem novas notificações criar toast azul canto inferior direito
- [X] Adicionar maneira de admin conseguir enviar uma mensagem para certos cargos, essa mensagem aparece nas notificações
- [X] Voltar a adicionar badges para funções na tabela dos utilizadores
- [X] Remover console.log inuteis
- [X] Settings page
- [X] Admin being able to edit user name, email, password
- [X] Melhorar navegação do website (botões para voltar para trás)
- [ ] Ver quem tem acesso ao servidor


## PT 2

- [X] Traduzir mensagem de alert de broadcast
- [X] Corrigir bug na seleção unica
- [X] Fazer com que se possa apagar paginas
- [X] Fazer com que o caminho do grupo pai seja mais facilmente editavel ao editar paginas
- [X] Corrigir bug quando se abre um datepicker o modal muda de posição
- [X] Meter alert de notificações bem destacado no centro da tela

- [X] Fazer com que o admin consiga apagar utilizadores
- [X] Fix texto em criar função, agora ao criar/editar uma função o texto é renderizado corretamente


- [X] Adicionar sistema de maração de férias administrador ao criar utilizador define dias por ano que o trabalhador ainda pode gozar.
  - [ ] Calendar Performance
  - [ ] Integração com feriados, provavelmente meter uma unica vez todos os feriados, utilizadores, tem que ter mais uma coluna de clinica, e feriados ha nacionais e locais, os locais so afetam os utilizadores que trabalham na clinica desse local
  - [ ] Replace default date inputs with custom datepicker
  - [X] Página especial (Sempre no menu) para férias
  - [X] Na página de marcação de férias vai ter um unico anual calendar (em que da para alterar o ano) em que utilizadores podem ver, mas não selecionar dias em que outras pessoas do mesmo cargo já tenham selecionado. Ou seja um trabalhador só pode ver ferias de pessoas com o mesmo cargo.
  - [X] Adicionar um campo nos cargos que é "é cargo de férias?", pessoas com cargos que tem este campo só podem ver/influenciar outra pessoas com o mesmo cargo
  - [X] Dias selecionados tem 3 tipos: Hover (ou seja não guardado), Solicitado (ou seja guardado mas não confirmado) e concedido (ou seja ferias que o administrador já verificou e aprovou)
  - [X] Administrador tem que ter uma página para a a gestão de férias, em que tem uma tabela inicial com todos os cargos de férias. Depois de clicar um cargo de férias, tem todos as solicitações de férias ordenadas por data.
  - [X] Administrador pode recusar e dar um motivo em especifico por ter recusado ou não.
  - [X] Notificar administrador quando há novas solicitações de férias
  - [X] Notificar colegas do mesmo cargo de férias quando há novas férias solicitadas


- [X] Duplicar página
- [X] Drag and drop ordem campos
- [X] Fix bug ao editar página ele volta por defeito raiz /
- [X] Lista drag and drop para o administrador ordenar páginas
- [X] Make button shape consistent (square)
- [X] Fix quando o grupo esta vazio para nao aparecer pagina de 404
- [X] Pagina para gerir paginas, aparecer primeiro so os grupo e depois clicar nos grupos para aparecer as paginas desses grupos
- [X] Fazer campo para dar upload de ficheiros ao criar paginas para icon
- [X] Adicionar pre visualização do icon da página no site
- [X] Criar novo tipo de permissão para adição (ou seja pode inserir nos campoes vazios de um registo, mas nao pode editar o que ja foi preenchido)
- [ ] Make the color of checkboxes consistent
- [X] Make the path input when creating a new page disabled
- [X] Adicionar botao para editar grupo no menu de paginas/grupos
- [X] Add static menu on admin menu





- [X] No menu estatico meter férias numa pasta de recursos humanos
- [ ] ~Fix texto em modal que sai fora do modal~
- [X] Nas ferias remover o conceito de "função de férias" e ao criar/editar uma função selecionar la funções que interferem nas férias
- [X] Ver bug que certos ficheiros ao guardar ficam com o utilmo caractere cortado
- [X] Ver bug que quando registo so tem data nao deixa submete
- [X] Admin menu começar todo fechado, e ver bug em que adiciona margem 
- [X] Novo tipo de campo, checkbox
- [X] Ver bug paginas aparecem no menu a users mesmo que nao tenham acesso 

- [ ] Ver bug de quando admin apaga utilizador, o utilizador se ainda tiver sesão nao é logado fora



- [ ] Ver alerta que não sai sozinho depois de criar um registo
- [ ] ~Meter no fim de cada registo um texto ou algum tipo de indicação para indicar que para aceder aos ficheiros tem se que clicar la~

- [X] Página especial (Sempre no menu) para o inicio
- [ ] Make inputs have the same hover effects
- [ ] ~Administrador ao criar utilizador submete num date picker quando é que o trabalhador nasceu.~
- [ ] Ver coisas giras para se meter na pagina inicial (editor de markdown para o admin, aniversários)
- [ ] Fazer com que ao editar uma pagina novos campos tenham um nome interno decente

- [ ] Logotipo para notificações
- [ ] Traduzir caminhos de paginas
- [ ] Performance

- [ ] Ver possivel bug quando o admin enviar uma mensagem/notifica utilizadores que o alert nao sai automaticamente




## PT 3

- [ ] Guardar logs de alteração, admin consegue ver quem editou X campo e quando
- [ ] Admin conseguir ver todas as férias de todos os grupos ou por grupo. No HPVC eles tem a cada dia identificado por codigo unico (ID de DB) cada pessoa que esta de ferias nesse dia. Ter uma legenda com o resumo das ferias de cada utilizador (quantos dias tirados e datas)
- [ ] Na pagina inicial, ter 3 partes, Novidades, Documentos, Protocolos e Beneficios
  - [ ] Editor de markdown para cada pagina
  - [ ] Ideias de coisas para documentos: Manual da qualidade, Regulamento Interno, Politicas, Direitos do trabalhor, Codigo de Conduta, Manual Novo colaboradores
  - [ ] Documentos tem que ter tomada de conhecimentos
- [ ] Escolher quem notificar em novo registo, ou seja, ao criar/editar pagina, ou notificar toda a gente que tem acesso ou entao uma função em especifico
    


## Referencias:
- [Bizneo](https://www.bizneo.com/pt-pt/gestao-ferias/)
