UI_LAYOUT.md
1. Estrutura Global
┌───────────────┐
│ Sidebar       │
│ - Biblioteca  │
│ - Calendário  │
│ - Legendas    │
│ - Estatísticas│
└───────▲───────┘
        │
┌───────┴───────────────┐
│ Main Content Area     │
└───────────────────────┘


Sidebar:

Sempre visível

Ícones + texto

Estado ativo claro

2. Tela: Biblioteca

Função: visão geral da coleção

Componentes:

Grid de capas

Indicador de progresso no card

Filtros:

Status

Temporada

Coleções

Interações:

Clique → Página do Anime

Hover → ações rápidas (opcional)

3. Tela: Página do Anime

Função: centro de controle do anime

Layout:

Header grande com capa

Informações:

Título

Sinopse

Status

Lista de episódios

Lista de episódios:

Número

Título (se existir)

Progresso visual

Estado (não visto / em progresso / concluído)

Ações:

Assistir

Selecionar legendas

Marcar manualmente

4. Tela: Player (Overlay)

Função: assistir sem distrair

Player externo (MPV)

UI mínima

Controles básicos

Progresso observado automaticamente

📌 Player nunca altera estado diretamente.

5. Tela: Calendário / Countdown

Função: visão temporal

Componentes:

Lista por temporada

Cada anime mostra:

Data do último episódio

Countdown visual

Filtros:

Temporada

Status

📌 Dados derivados do AniList
📌 Nunca fonte de verdade

6. Tela: Legendas

Função: gerenciar transformações

Componentes:

Lista de legendas detectadas

Seleção por anime / episódio

Painel de estilos:

Fonte

Outline

Shadow

Tamanho

Fluxo:

Seleciona episódios

Aplica transformação

Gera nova versão

Histórico preservado

7. Tela: Estatísticas

Função: feedback, não controle

Tempo assistido

Episódios concluídos

Evolução temporal

📌 Estatísticas nunca alteram domínios.