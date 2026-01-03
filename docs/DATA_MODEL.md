AnimeHub — Modelo de Dados Canônico

Este documento define quais dados existem,
como se relacionam,
e quais estados são válidos no AnimeHub.

Ele é o reflexo direto e obrigatório de:

DOMAIN_CONTRACTS.md

EVENT_MAP.md

SERVICE_CONTRACTS.md

Se algo não estiver aqui, não existe no sistema.

0. PRINCÍPIOS DO MODELO DE DADOS
0.1 Fonte de verdade

Entidades primárias:

Anime

Episode

File

Subtitle

Collection

Entidades derivadas:

Statistics

Eventos:

Podem ser persistidos

Nunca substituem estado primário

0.2 Identidade e mutabilidade

Toda entidade possui:

ID interno imutável

IDs externos:

São auxiliares

Nunca substituem identidade interna

Nenhum dado crítico é sobrescrito sem:

Versionamento

Histórico preservado

0.3 Relações explícitas

Não existem relações implícitas

Nenhuma inferência automática é permitida

1. ENTIDADE: ANIME
Identidade

anime_id (interno, imutável)

Campos conceituais

titulo_principal

titulos_alternativos

tipo (TV | Movie | OVA | Special)

status (em_exibicao | finalizado | cancelado)

total_episodios (opcional)

data_inicio (opcional)

data_fim (opcional)

metadados_livres

criado_em

atualizado_em

Relações

1:N → Episode

N:M → Collection

1:N → ExternalReference

1:N → AnimeAlias

Estados válidos

Anime pode existir:

Sem episódios

Sem arquivos

Sem referências externas

2. ENTIDADE: EPISODE
Identidade

episode_id (interno)

Campos conceituais

anime_id (obrigatório)

numero (inteiro ou especial)

titulo (opcional)

duracao_esperada (opcional)

progresso_atual

estado (nao_visto | em_progresso | concluido)

criado_em

atualizado_em

Relações

N:1 → Anime

1:N → File (associados)

0..1 → File (arquivo_principal)

Estados válidos

Episode:

Nunca existe sem Anime

Pode existir sem arquivo

Assume uma versão prática

3. ENTIDADE: FILE
Identidade

file_id

Campos conceituais

caminho_absoluto

tipo (video | legenda | imagem | outro)

tamanho

hash (opcional)

data_modificacao

origem (scan | importacao | manual)

criado_em

atualizado_em

Relações

N:M → Episode

1:N → Subtitle (quando tipo = legenda)

Estados válidos

File pode:

Não estar associado a nada

Estar associado a múltiplos episódios (exceção)

Arquivo físico:

Nunca é deletado automaticamente

Nunca é sobrescrito

4. ENTIDADE: SUBTITLE
Identidade

subtitle_id

Campos conceituais

file_id (origem)

formato (SRT | ASS | VTT)

idioma

versao

eh_original (boolean)

criado_em

Relações

N:1 → File

1:N → SubtitleTransformation

Estados válidos

Toda Subtitle:

Possui origem

Nunca substitui outra

Pode gerar infinitas derivadas

5. ENTIDADE: SUBTITLE_TRANSFORMATION
Identidade

transformation_id

Campos conceituais

subtitle_id_origem

tipo (style | timing | conversao)

parametros_aplicados

criado_em

Relações

N:1 → Subtitle

6. ENTIDADE: COLLECTION
Identidade

collection_id

Campos conceituais

nome

descricao

criado_em

Relações

N:M → Anime

Estados válidos

Coleções:

Não alteram Anime

Não afetam progresso

São puramente organizacionais

7. ENTIDADE: EXTERNAL_REFERENCE
Identidade

external_reference_id

Campos conceituais

anime_id

fonte (AniList)

external_id

criado_em

Relações

N:1 → Anime

Estados válidos

ExternalReference:

Nunca substitui dados locais

Pode ser removida sem impacto estrutural

8. ENTIDADE: ANIME_ALIAS
Identidade

alias_id

Campos conceituais

anime_principal_id

anime_alias_id

criado_em

Relações

2 × Anime

Estados válidos

AnimeAlias:

Nunca é deletado

Mantém histórico de merges

9. ENTIDADE: STATISTICS_SNAPSHOT (DERIVADA)
Identidade

snapshot_id

Campos conceituais

tipo (global | por_anime | por_periodo)

valor

gerado_em

⚠️ Nunca é fonte primária de verdade

10. EVENT STORE (OPCIONAL, RECOMENDADO)
ENTIDADE: DOMAIN_EVENT

Campos conceituais:

event_id

tipo_evento

payload

ocorrido_em

Uso

Auditoria

Debug

Replay de eventos

Análise de falhas

11. REGRAS DE INTEGRIDADE (CRÍTICAS)
Estados proibidos

Episode sem Anime

Subtitle sem File

File deletado automaticamente

Estatística como fonte primária

Merge sem AnimeAlias

Se ocorrer → erro arquitetural grave.

12. MATRIZ FINAL DE RELAÇÕES (RESUMO)
Entidade	Relacionamentos
Anime	Episode, Collection, ExternalReference, AnimeAlias
Episode	Anime, File
File	Episode, Subtitle
Subtitle	File, Transformation
Collection	Anime
Statistics	(derivada)
13. ESTADO FINAL DO PROJETO

Domínios: FECHADOS

Eventos: FECHADOS

Serviços: FECHADOS

Modelo de Dados: FECHADO

Ambiguidade: ZERO