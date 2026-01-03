AnimeHub — Documento Canônico de Continuidade

Este arquivo é o ponto de entrada obrigatório para qualquer IA ou humano.
Ele define:

o que é o projeto

o que NÃO é

em que estágio ele está

como continuar a implementação sem reinventar nada

1. O QUE É O ANIMEHUB

AnimeHub é um gerenciador local, offline-first, de animes japoneses (incluindo filmes), com foco em:

Controle explícito de biblioteca

Progresso de visualização confiável

Transformações não destrutivas de legendas

Robustez estrutural acima de conveniência

Manutenção mínima a longo prazo

Não é um player.
Não é um scraper automático.
Não é um sistema de streaming.

2. PRINCÍPIOS INEGOCIÁVEIS

Nada é implícito

Nada é destruído automaticamente

Toda transformação gera histórico

Eventos coordenam, serviços não se chamam

Dados derivados nunca são fonte de verdade

Implementação nunca decide arquitetura

Violação de qualquer um → erro crítico.

3. DOCUMENTOS CANÔNICOS (ORDEM DE LEITURA)

README.md (este arquivo)

DOMAIN_CONTRACTS.md

EVENT_MAP.md

SERVICE_CONTRACTS.md

DATA_MODEL.md

👉 Código deve obedecer aos documentos, nunca o contrário.

4. ESCOPO FUNCIONAL
Incluído

Animes japoneses (TV, Movie, OVA, Special)

Biblioteca local

Player externo (MPV)

Estilo e timing de legendas

Progresso persistente

Excluído (por design)

Séries ocidentais

Streaming online

Download automático

Recomendações algorítmicas

Edição destrutiva de arquivos

5. ESTADO ATUAL DO PROJETO
Camada	Estado
Visão	Fechada
Domínios	Fechados
Eventos	Fechados
Serviços	Fechados
Dados	Fechados
Código	NÃO iniciado

📌 Este é o único momento seguro para iniciar código.

6. COMO UMA IA DEVE CONTINUAR O PROJETO
Ordem obrigatória de implementação

Estrutura base do projeto

Domínios (entidades + invariantes)

Event bus interno

Persistência (repositórios)

Serviços

Integração com player

Interface

⚠️ Pular etapas quebra o sistema.

7. EXPECTATIVA DE QUALIDADE DE CÓDIGO

Código:

Determinístico

Testável

Sem efeitos colaterais implícitos

Nenhum:

Mock permanente

Placeholder

“Depois a gente arruma”

8. SOBRE TECNOLOGIA (DELIBERADAMENTE ABERTO)

Linguagem: indiferente

Framework: indiferente

Banco: indiferente

UI: indiferente

📌 A arquitetura independe da stack.

9. COMO ENTREGAR ESTE PROJETO A OUTRA IA

Ao iniciar uma nova conversa:

Envie todos os arquivos .md

Diga:

“Leia todos os documentos. Não proponha mudanças arquiteturais. Apenas implemente.”

Se a IA:

questionar contratos → erro

sugerir simplificação → erro

propor automação implícita → erro

10. GARANTIA DE CONTINUIDADE

Qualquer IA, lendo estes arquivos, consegue:

Entender o projeto inteiro

Saber exatamente em que fase ele está

Continuar sem decisões subjetivas

Manter o mesmo nível de rigor

11. STATUS FINAL

Projeto completamente especificado.
Zero ambiguidade.
Pronto para implementação.