AnimeHub — Contratos de Domínio Canônicos
Versão do Documento: 1.0
Data de Criação: 24/12/2025
Descrição Geral: Este documento define O QUE o sistema AnimeHub é. Ele estabelece contratos de domínio canônicos, focando em regras imutáveis, entidades, invariantes e ações permitidas/proibidas. Não define tecnologia, implementação ou UI. Qualquer código que viole este documento está arquiteturalmente errado, mesmo que funcione funcionalmente.

0. REGRAS GLOBAIS (IMUTÁVEIS)
0.1 Natureza do Sistema

O AnimeHub é local-first.
O usuário controla os arquivos.
O sistema nunca destrói dados silenciosamente.

0.2 Verdades Fundamentais

Toda entidade possui:
Identidade interna imutável.

IDs externos:
São auxiliares.
Nunca substituem identidade interna.


0.3 Estados Proibidos (NUNCA Podem Existir)

Episódio sem Anime.
Legenda sem arquivo de origem.
Progresso negativo.
Progresso maior que duração.
Estatística como fonte primária.
Arquivo deletado automaticamente.

Consequência: Se qualquer um ocorrer → erro crítico de arquitetura.

1. DOMÍNIO: ANIME
1.1 Escopo
Representa exclusivamente obras japonesas:

Séries de TV.
Filmes.
OVAs.
Especiais.

⚠️ Não Representa:

Donghua.
Animação ocidental.
Conteúdo genérico.

1.2 Entidade: Anime
Campos Conceituais:

id (identidade interna).
Título principal.
Títulos alternativos.
Tipo (TV | Movie | OVA | Special).
Status (em_exibição | finalizado | cancelado).
Total de episódios (opcional).
Datas relevantes (opcionais).
Metadados livres (gêneros, estúdio, etc.).

1.3 Invariantes

Um Anime:
Pode existir sem episódios.
Pode existir sem arquivos.
Pode existir sem fonte externa.

Identidade nunca muda.
Duplicatas são permitidas até resolução explícita.

1.4 Ações Permitidas

Criar manualmente.
Criar via importação.
Atualizar metadados.
Associar fontes externas.
Marcar duplicata.
Fundir duplicatas (manual).

1.5 Ações Proibidas

Deletar automaticamente.
Fundir sem confirmação.
Alterar histórico ao editar metadados.


2. DOMÍNIO: EPISODE
2.1 Escopo
Representa uma unidade de exibição pertencente a um Anime.
2.2 Entidade: Episode
Campos Conceituais:

id (identidade interna).
anime_id (referência ao Anime).
Número (regular ou especial).
Título (opcional).
Duração esperada (opcional).
Progresso atual.
Estado (não_visto | em_progresso | concluído).

2.3 Invariantes

Todo episódio pertence a exatamente um Anime.
Um episódio:
Pode existir sem arquivo.
Assume uma versão prática.

Progresso:
Nunca diminui automaticamente.
Nunca ultrapassa duração.


2.4 Ações Permitidas

Criar manualmente.
Criar via automação explícita.
Associar arquivos.
Atualizar progresso.
Marcar como especial.

2.5 Ações Proibidas

Episódio órfão.
Reset implícito de progresso.
Merge automático silencioso.


3. DOMÍNIO: FILE (ARQUIVO FÍSICO)
3.1 Escopo
Representa arquivos reais no disco.
3.2 Entidade: File
Campos Conceituais:

id (identidade interna).
Caminho absoluto.
Tipo (vídeo | legenda | imagem | outro).
Tamanho.
Hash (opcional).
Data de modificação.
Origem (scan | importação | manual).

3.3 Invariantes

Arquivo:
Nunca é assumido confiável.
Pode mudar de caminho.

Nome do arquivo não é verdade absoluta.
Arquivo original nunca é sobrescrito.

3.4 Ações Proibidas

Deletar fisicamente.
Reescrever conteúdo.
Inferir associação sem confirmação.


4. DOMÍNIO: SUBTITLE
4.1 Escopo
Legendas como dados transformáveis, não apenas arquivos.
4.2 Entidade: Subtitle
Campos Conceituais:

id (identidade interna).
file_id de origem.
Formato (SRT | ASS | VTT).
Idioma.
Versão.
é_original (boolean).

4.3 Invariantes

Toda legenda:
Possui origem.
Nunca substitui outra.

Transformações:
São versionadas.
São reversíveis.


4.4 Ações Permitidas

Aplicar estilo (fonte, outline, tamanho).
Ajustar timing ocasionalmente.
Converter formato.
Processar em batch.

4.5 Ações Proibidas

Edição destrutiva.
Sobrescrever arquivo original.


5. DOMÍNIO: COLLECTION
5.1 Escopo
Agrupamento puramente organizacional.
5.2 Invariantes

Coleções:
Não afetam progresso.
Não alteram Anime.

Anime pode pertencer a várias coleções.


6. DOMÍNIO: STATISTICS
6.1 Escopo
Dados derivados, nunca primários.
6.2 Invariantes

Estatísticas:
Podem ser recalculadas.
Podem ser descartadas.

Nunca alteram domínios.


7. INTEGRAÇÕES EXTERNAS
7.1 AniList

Fonte auxiliar.
Nunca autoritária.
Nunca sobrescreve dados locais.

7.2 Player (MPV)

Observável.
Controlável.
Falhas não corrompem estado.


8. DECLARAÇÃO FINAL
Se algo:

Não está descrito aqui → não deve ser implementado.

Este documento:

Precede o código.
Precede o banco.
Precede qualquer IA.