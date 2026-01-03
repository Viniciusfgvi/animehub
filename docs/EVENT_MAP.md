AnimeHub — Mapa Canônico de Eventos e Fluxos

Este documento define o comportamento do sistema ao longo do tempo.
Ele responde, de forma inequívoca, à pergunta:
“Quando X acontece, o que o sistema faz?”

Não descreve UI.
Não descreve tecnologia.
Descreve causalidade e reação.

0. PRINCÍPIOS DO EVENT MAP
0.1 Evento é um fato, não uma intenção

Um evento:

Já aconteceu

É verdadeiro

Não depende de interface

Exemplos:

❌ “Usuário clicou no botão”

✅ “Arquivo de vídeo foi detectado”

✅ “Episódio foi marcado como concluído”

0.2 Tipos de eventos
Evento Primário

Fato originado por:

Ação explícita do usuário

Interação com o mundo externo (scan, player)

Evento Derivado

Consequência lógica de outro evento.

Evento Observável

Evento que existe para:

Atualizar UI

Atualizar estatísticas

Atualizar cache

⚠️ Eventos observáveis nunca alteram domínios diretamente.

0.3 Regra estrutural absoluta

Nenhum evento pode alterar diretamente mais de um domínio.

Coordenação acontece por reações, nunca por acoplamento.

1. EVENTO: SCAN DE DIRETÓRIO
1.1 Evento primário

DirectoryScanned

Emitido quando:

O sistema termina de varrer um diretório configurado

1.2 Eventos derivados
FileDetected

Emitido para cada arquivo relevante encontrado:

Vídeo

Legenda

Imagem associável

1.3 Reações por domínio
Domínio: File

Registra o arquivo

Atualiza:

tamanho

data de modificação

hash (se configurado)

Não associa automaticamente

Domínio: Anime / Episode

Nenhuma ação direta

Associação exige evento explícito posterior

📌 Decisão canônica
Scan nunca cria Anime ou Episode automaticamente.

2. EVENTO: CRIAÇÃO DE ANIME
2.1 Evento primário

AnimeCreated

Origem:

Criação manual

Importação explícita

2.2 Reações
Domínio: Anime

Cria entidade Anime

Estado inicial neutro

Domínio: Statistics

Nenhuma reação imediata

3. EVENTO: CRIAÇÃO DE EPISÓDIO
3.1 Evento primário

EpisodeCreated

Pré-condição:

Anime existente

3.2 Reações
Domínio: Episode

Cria episódio

Estado inicial: não_visto

Progresso = 0

4. EVENTO: ASSOCIAÇÃO DE ARQUIVO A EPISÓDIO
4.1 Evento primário

FileLinkedToEpisode

Origem:

Ação manual do usuário

Automação explícita e confirmada

4.2 Reações
Domínio: Episode

Registra associação

Define papel do arquivo:

principal (vídeo)

auxiliar (legenda, extra)

Domínio: File

Mantém referência reversa

4.3 Evento derivado
EpisodeBecamePlayable

Emitido quando:

Episódio passa a ter arquivo de vídeo válido

5. EVENTO: INÍCIO DE REPRODUÇÃO
5.1 Evento primário

PlaybackStarted

Emitido quando:

Player inicia reprodução de um episódio

5.2 Reações
Domínio: Episode

Estado → em_progresso

Domínio: Statistics

Registra sessão temporária

📌 Nenhum progresso é persistido aqui.

6. EVENTO: ATUALIZAÇÃO DE PROGRESSO
6.1 Evento primário

PlaybackProgressUpdated

Emitido periodicamente pelo player.

6.2 Invariantes

Progresso:

Nunca diminui automaticamente

Nunca ultrapassa duração conhecida

6.3 Reações
Domínio: Episode

Atualiza progresso atual

Domínio: Statistics

Atualiza métricas derivadas

7. EVENTO: FINALIZAÇÃO DE EPISÓDIO
7.1 Evento primário

EpisodeCompleted

Emitido quando:

Progresso atinge limiar configurado (ex: ≥ 90%)

7.2 Reações
Domínio: Episode

Estado → concluído

Domínio: Anime

Atualiza contadores derivados (ex: episódios assistidos)

Domínio: Statistics

Atualiza totais globais

8. EVENTO: DETECÇÃO DE LEGENDA
8.1 Evento primário

SubtitleDetected

Origem:

Scan de diretório

Importação manual

8.2 Reações
Domínio: Subtitle

Registra legenda

Detecta:

formato

idioma

Domínio: File

Mantém arquivo original imutável

9. EVENTO: APLICAÇÃO DE ESTILO DE LEGENDA
9.1 Evento primário

SubtitleStyleApplied

Origem:

Ação manual

Batch explícito

9.2 Invariantes

Legenda original:

Nunca é sobrescrita

Sempre gera nova versão

9.3 Reações
Domínio: Subtitle

Cria nova versão

Registra transformação de estilo

10. EVENTO: AJUSTE DE TIMING DE LEGENDA
10.1 Evento primário

SubtitleTimingAdjusted

10.2 Reações
Domínio: Subtitle

Gera nova versão

Registra transformação de timing

📌 Timing é tratado como transformação, não edição destrutiva.

11. EVENTO: FUSÃO DE ANIMES (DUPLICATAS)
11.1 Evento primário

AnimeMerged

Pré-condições:

Ação manual

Confirmação explícita

11.2 Reações
Domínio: Anime

Um anime torna-se principal

Outro vira alias histórico

Domínio: Episode

Episódios são reassociados explicitamente

12. EVENTO: REBUILD DE ESTATÍSTICAS
12.1 Evento primário

StatisticsRebuilt

Origem:

Ação manual

Manutenção

12.2 Reações
Domínio: Statistics

Recalcula todos os dados derivados

📌 Nenhum domínio primário é alterado.

13. EVENTOS PROIBIDOS (CRÍTICOS)

Os seguintes eventos NUNCA DEVEM EXISTIR:

AutoAnimeDeleted

SubtitleOverwritten

ImplicitEpisodeMerge

ProgressResetWithoutUserAction

FileAutoDeleted

Se aparecerem → erro arquitetural grave.

14. GARANTIA DE CONTINUIDADE

Qualquer IA que leia:

DOMAIN_CONTRACTS.md

EVENT_MAP.md

Consegue:

Entender o sistema

Implementar sem improvisar

Evoluir sem quebrar contratos

15. ESTADO DO PROJETO

Domínios: FECHADOS

Eventos: FECHADOS

Serviços: DEFINIDOS

Código: NÃO INICIADO

