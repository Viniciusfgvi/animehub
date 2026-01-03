AnimeHub — Contratos Canônicos de Serviços

Este documento define QUEM FAZ O QUÊ no sistema.
Ele impede:

serviços genéricos

lógica escondida

acoplamento indevido

Qualquer serviço que viole este contrato está arquiteturalmente errado.

0. REGRAS GLOBAIS DOS SERVIÇOS
0.1 Definição de Serviço

Um Serviço:

Orquestra domínios

Reage a eventos

Não possui estado durável próprio

📌 Estado durável pertence exclusivamente aos domínios.

0.2 O que um Serviço NÃO É

❌ Controller de UI

❌ Wrapper de banco

❌ Lugar para “resolver tudo”

0.3 Comunicação entre serviços

Serviços:

Consomem eventos

Emitem eventos

Serviços:

Nunca chamam outros serviços diretamente

Nunca compartilham estado

Coordenação ocorre por eventos, não por chamadas diretas.

1. ANIME SERVICE
Responsabilidade

Gerenciar o ciclo de vida do Anime como entidade de domínio.

Pode fazer

Criar Anime

Atualizar metadados

Associar fontes externas

Resolver duplicatas (merge manual)

NÃO pode fazer

Criar episódios automaticamente

Criar arquivos

Alterar progresso de episódios

Calcular estatísticas

Eventos que consome

AnimeCreated

ExternalMetadataLinked

AnimeMerged

Eventos que emite

AnimeUpdated

AnimeMerged

2. EPISODE SERVICE
Responsabilidade

Gerenciar episódios e progresso de visualização.

Pode fazer

Criar episódios

Atualizar progresso

Marcar como concluído

Associar arquivos existentes

NÃO pode fazer

Criar Anime

Criar arquivos físicos

Manipular legendas

Gerar estatísticas globais

Eventos que consome

EpisodeCreated

PlaybackStarted

PlaybackProgressUpdated

EpisodeCompleted

FileLinkedToEpisode

Eventos que emite

EpisodeBecamePlayable

EpisodeProgressUpdated

EpisodeCompleted

3. FILE SERVICE
Responsabilidade

Gerenciar arquivos físicos como entidades observáveis.

Pode fazer

Registrar arquivos detectados

Atualizar metadados físicos

Calcular hash

Detectar alterações

NÃO pode fazer

Criar Anime ou Episódio

Decidir associações implicitamente

Deletar arquivos físicos

Eventos que consome

DirectoryScanned

Eventos que emite

FileDetected

FileUpdated

FileLinkedToEpisode

4. SUBTITLE SERVICE
Responsabilidade

Orquestrar transformações de legendas.

Pode fazer

Converter formatos

Aplicar estilos

Ajustar timing

Criar versões derivadas

NÃO pode fazer

Sobrescrever legenda original

Alterar arquivos físicos diretamente

Modificar progresso de episódios

Eventos que consome

SubtitleDetected

SubtitleStyleApplied

SubtitleTimingAdjusted

Eventos que emite

SubtitleVersionCreated

SubtitleProcessed

5. PLAYBACK SERVICE
Responsabilidade

Integrar o sistema com o player de mídia (MPV).

Pode fazer

Iniciar reprodução

Pausar e parar

Monitorar progresso

Emitir eventos de playback

NÃO pode fazer

Persistir progresso

Alterar estado de domínio diretamente

Criar entidades

Eventos que consome

PlaybackRequested

Eventos que emite

PlaybackStarted

PlaybackProgressUpdated

PlaybackStopped

6. STATISTICS SERVICE
Responsabilidade

Gerar dados derivados e agregados.

Pode fazer

Agregar dados

Cachear resultados

Recalcular estatísticas

NÃO pode fazer

Alterar domínios

Criar entidades

Persistir dados como fonte primária

Eventos que consome

EpisodeCompleted

PlaybackProgressUpdated

StatisticsRebuilt

Eventos que emite

StatisticsUpdated

7. EXTERNAL INTEGRATION SERVICE
Responsabilidade

Integrar serviços externos (AniList).

Pode fazer

Buscar metadados externos

Normalizar dados recebidos

Emitir eventos de associação

NÃO pode fazer

Criar Anime automaticamente

Substituir dados locais

Ser fonte de verdade

Eventos que consome

ExternalMetadataRequested

Eventos que emite

ExternalMetadataFetched

ExternalMetadataLinked

8. SERVIÇOS PROIBIDOS (ANTI-PATTERNS)

Os seguintes NÃO DEVEM EXISTIR:

LibraryService

ManagerService

UtilsService

GodService

Se aparecerem → falha arquitetural.

9. MATRIZ DE RESPONSABILIDADE (RESUMO)
Serviço	Pode	Não pode
Anime	Metadados	Progresso
Episode	Progresso	Arquivos
File	Detectar	Associar implícito
Subtitle	Transformar	Destruir
Playback	Observar	Persistir
Statistics	Derivar	Alterar
External	Buscar	Mandar
10. ESTADO DO PROJETO APÓS ESTE DOCUMENTO

Domínios: FECHADOS

Eventos: FECHADOS

Serviços: FECHADOS

Ambiguidade: ZERO