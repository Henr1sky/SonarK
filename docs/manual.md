# Manual de uso do SonarK

Este manual documenta a versão **0.0.10**. O [catálogo](catalogo.md) indica o que cada ação lê ou produz.

## Índice

- [Instalação e requisitos](#instalação-e-requisitos)
- [Projetos e arquivos](#projetos-e-arquivos)
- [Atalhos por tela](#atalhos-por-tela)
- [Analisar uma captura](#analisar-uma-captura)
- [Ler um log e convertê-lo para TShark](#ler-um-log-e-convertê-lo-para-tshark)
- [Consultar um PDF](#consultar-um-pdf)
- [Editar comandos e tratar a saída](#editar-comandos-e-tratar-a-saída)
- [Histórico, exportação e armazenamento](#histórico-exportação-e-armazenamento)
- [Escopo e limites](#escopo-e-limites)
- [Resolver problemas](#resolver-problemas)

## Instalação e requisitos

O SonarK roda em Linux e precisa de um terminal interativo. Compilação e execução dependem de ferramentas diferentes. Use as verificações da tabela para identificar o que já está disponível no seu ambiente.

| Uso | Dependências | Verificação |
| --- | --- | --- |
| Clonar e compilar | Git, Cargo e Rust com suporte à edição 2024 | `git --version`, `cargo --version`, `rustc --version` |
| Ler capturas com os presets | `tshark`; `capinfos` para metadados e hashes | `tshark --version`, `capinfos --version` |
| Converter logs em PCAPNG | `text2pcap` com suporte a PDUs exportadas | `text2pcap -h` e procure a opção `-P` |
| Analisar PDFs anexados | `pdftotext`, `pdfinfo` e `pdfimages` do Poppler | `command -v pdftotext pdfinfo pdfimages` |
| Usar o editor de etapas | `sort`, `uniq`, `head`, `tail`, `grep`, `cut`, `wc`, `awk` | `command -v sort uniq head tail grep cut wc awk` |

### Instalar o binário para uso diário

Em Kali, Debian e derivados com esses pacotes disponíveis, instale as dependências do sistema pelo gerenciador da distribuição:

```bash
sudo apt update
sudo apt install git build-essential cargo rustc tshark wireshark-common poppler-utils coreutils grep mawk
```

Verifique `rustc --version` e `cargo --version` antes de compilar. Se a distribuição fornecer uma versão sem suporte à edição 2024, atualize a toolchain. O conjunto de presets desta versão foi testado com TShark **4.6.6**; dissectors e campos podem variar em versões diferentes.

```bash
# Baixa o código-fonte do SonarK para uma pasta local.
git clone https://github.com/Henr1sky/SonarK.git

# Entra na pasta do projeto baixado.
cd SonarK

# Compila e instala o binário usando as versões fixadas em Cargo.lock.
cargo install --locked --path .

# Confirma que o binário está acessível e mostra a versão instalada.
sonark --version

# Abre a interface no projeto investigacao com a captura indicada.
sonark --project investigacao "/caminho/captura.pcapng"
```

Com a configuração padrão do Cargo, o executável fica em `~/.cargo/bin/sonark`. Se `sonark` não for encontrado, acrescente o diretório ao `PATH` do seu shell. Na sessão atual:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Para atualizar a partir de um clone local, rode `git pull --ff-only` nessa pasta e repita `cargo install --locked --path .`. Para remover apenas o executável instalado pelo Cargo, use `cargo uninstall sonark`. Os dados dos projetos ficam na pasta descrita em [armazenamento](#histórico-exportação-e-armazenamento).

### Executar sem instalar

```bash
cargo run --locked
cargo build --locked --release
cargo test --locked
```

Para rodar o binário otimizado, use `./target/release/sonark`. A CLI aceita `--help`, `--version`, `--project NOME` e um arquivo local opcional. A primeira compilação pode precisar da rede para baixar dependências do Cargo. Com o cache preparado, acrescente `--offline` ao comando do Cargo.

```bash
cargo run --locked --offline -- --project lab-dns "/caminho/arquivo.pcapng"
cargo run --locked --offline -- --project lab-http "/caminho/access.log"
```

## Projetos e arquivos

Sem `--project`, a TUI usa o projeto `default`. A entrada principal é um arquivo local. O projeto pode guardar referências adicionais, como notas externas, JSON, CSV e PDFs.

1. No menu, vá a **[23] Workspace & Projects**.
2. Pressione **n** para listar projetos. **Enter** abre o selecionado. **c** inicia um novo.
3. Dê um nome ao projeto. Use até 64 caracteres ASCII entre letras, dígitos, hífens e sublinhados.
4. Na tela de configuração, use **p** para vincular a entrada e **a** para anexar arquivos de apoio. **n** edita o nome. **x** limpa os arquivos da configuração em andamento.
5. Pressione **Enter** para criar ou **Esc** para cancelar.

Em um projeto já aberto, **p** muda a entrada, **u** lista referências e **m** edita a nota. Na lista de referências, **a** adiciona um arquivo e **Enter** abre os arquivos de texto no leitor. **r** no Workspace renomeia o projeto e **Enter** alterna inglês e português. A escolha de idioma é persistida na configuração local.

O navegador mostra a pasta atual no título. Use **←** para subir ao diretório pai. Para digitar ou colar um caminho diferente, pressione **t**, use **Ctrl+U** para apagar o caminho preenchido, cole o novo e confirme.

## Atalhos por tela

As letras são sensíveis a maiúsculas onde a tabela distingue **r** de **R**, **b** de **B** ou **e** de **E**.

| Tela | Teclas e comportamento |
| --- | --- |
| Menu e listas | **↑/↓** ou **k/j** navegam. **Enter**, **→** ou **l** abrem a seleção onde disponível. **Esc** volta em telas internas. |
| Categoria | **/** busca no título ou ID. Use o ID na busca para achar uma ação numerada. **i** mostra os IDs do inventário ainda sem preset. |
| Geral | **F1** abre o guia, **?** mostra a ajuda rápida e **q** sai. **Ctrl+C** cancela uma execução ativa ou sai quando não há execução. |
| Navegador | **Enter** ou **→** abre a seleção. **←** ou **Backspace** sobe uma pasta. **d** abre Downloads, **~** vai para a pasta pessoal, **w** vai à pasta de trabalho, **a** alterna arquivos ocultos e **t** edita o caminho. |
| Campo de texto | **←/→**, **Home** e **End** movem o cursor. **Backspace** e **Delete** apagam. **Ctrl+U** limpa. **Enter** confirma e **Esc** cancela. |
| Ação | **Enter** executa. **h** abre a ajuda da ação, **e** alterna a explicação no painel, **a** abre ajustes, **s** salva o preset, **b** carrega e **f** alterna favorito. |
| Ação com editor | **t** abre as etapas de saída, **c** abre o comando completo e **R** restaura o original. No painel de comando, **r** alterna original e editado, e **c** copia o comando exibido. |
| Execução | **B** deixa a execução em segundo plano. Reabra a ação para acompanhá-la. **Esc** cancela a execução em primeiro plano; **Ctrl+C** pode cancelar a execução ativa em segundo plano. |
| Resultado | **↑/↓**, **PgUp/PgDn** e **←/→** rolam a saída. **x** abre os formatos de exportação. |
| Leitor de texto | **/** busca, **↑/↓** e **←/→** rolam, **o** abre outro arquivo ou volta à lista de referências, conforme a origem. **Esc** fecha. |
| Workspace | **n** projetos, **r** renomear, **m** nota, **u** referências, **y** histórico, **f** favoritos, **v** alternar log e captura derivada, **Enter** idioma. |

## Analisar uma captura

### Reconhecimento e DNS

1. Vincule o PCAP ou PCAPNG com **p**.
2. Abra **[1] Reconhecimento & Inventário** e execute **[1.1] Hierarquia de protocolos**. Use a saída para identificar quais protocolos foram dissecados.
3. Se houver DNS visível, abra **[10.1] Consultas DNS**. Para limitar a análise a uma janela de tempo, acrescente o filtro abaixo com **a → f**.

```text
frame.time_relative >= 10 && frame.time_relative <= 60
```

O filtro adicional é combinado com o filtro do preset. Uma saída vazia indica que a operação não produziu linhas naquelas condições. Verifique o filtro, o intervalo de tempo, a captura selecionada e o protocolo dissecado antes de concluir sobre o tráfego.

### Contar portas de destino dos SYN

A ação **[3.7]** lê `tcp.dstport` onde `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Abra **a → 1** para definir a origem e reabra **a → 2** para definir o destino. A receita padrão ordena portas iguais lado a lado, conta as ocorrências, ordena pelas contagens e mostra as dez primeiras. A coluna da esquerda é a quantidade de SYN observados e a da direita é a porta.

Para ver os respondentes, use **[2.2]**. Essa ação conta **SYN+ACK** pelo endereço de origem da resposta. A interpretação depende do recorte da captura, da direção e dos filtros aplicados.

### Ler payload TCP por porta e direção

Na ação **[4.6]**, use **a → p** para especificar a porta. Reabra **a** e use **r** para escolher o sentido, **u** para restringir a PSH e **d** para exibir texto com IP de origem e portas. **Para a porta** filtra `tcp.dstport`, **Da porta** filtra `tcp.srcport` e **Ambas** aceita os dois sentidos.

A ação decodifica cada segmento capturado. Para inspecionar um stream, use **[4.8]** para TCP ou **[4.9]** para UDP e informe o número em **a → s**. O que pode ser lido depende dos bytes presentes e da dissecação. Uma captura truncada ou um payload cifrado limita o conteúdo visível.

## Ler um log e convertê-lo para TShark

### Consultar diretamente

Crie um log de exercício com endereços reservados para documentação:

```bash
cat > /tmp/sonark-exemplo.log <<'LOG'
192.0.2.10 - - [18/Nov/2026:10:00:00 -0300] "GET /inicio HTTP/1.1" 200 42 "-" "SonarK-Lab"
192.0.2.11 - - [18/Nov/2026:10:00:05 -0300] "GET /ausente HTTP/1.1" 404 15 "-" "SonarK-Lab"
LOG
cargo run --locked -- --project lab-http /tmp/sonark-exemplo.log
```

Pressione **g** para abrir o leitor e use **/** para buscar texto sem distinguir maiúsculas de minúsculas. Para obter resultados exportáveis, abra **[1.4]** para resumo, **[1.12]** para requisições por cliente, **[5.7]** para status ou **[21.4]** para a linha do tempo. Essas ações analisam registros no formato HTTP Common ou Combined. No arquivo de exemplo, a contagem tem uma ocorrência de `200` e uma de `404`.

### Gerar uma captura derivada

1. Com o log vinculado, abra **[19.19]** e pressione **Enter**. O SonarK prepara os dados e chama `text2pcap`.
2. A saída mostra a pasta que contém `log-derived.pcapng` e `source.json`. O manifesto registra a origem, a representação, a contagem de registros e PDUs e o tratamento aplicado aos horários.
3. Pressione **o** no resultado para ativar essa captura como entrada dos presets TShark. **g** continua abrindo o log original, e **v** no Workspace alterna entre as entradas.
4. Escolha a ação compatível com a representação gerada.

| Formato detectado | O que vai para o PCAPNG | Como consultar |
| --- | --- | --- |
| HTTP Common ou Combined com horários reconhecidos | PDUs HTTP sintéticas. Métodos e URIs válidos alimentam requisições; os códigos registrados alimentam respostas. O horário provém do evento do log. | **[5.6]** métodos, **[5.7]** status ou campos HTTP pelo TShark. |
| JSON ou JSONL compatível | Objetos ou arrays como PDUs para o dissector JSON. | **[19.22]** chaves e caminhos. |
| Texto genérico ou CSV | Conteúdo do arquivo como dados genéricos, com divisão em blocos quando necessário. | **[19.21]**. Abra **a** e pressione **d** para visualizar os dados como texto. |

Use **[19.20]** para preservar os bytes como dados genéricos sem a detecção automática de formato. Na representação genérica e JSON, os horários do PCAPNG são gerados na conversão. O limite atual é **64 MiB** por arquivo local de texto. Um PCAPNG derivado de log HTTP não adquire IPs, portas, flags TCP, duração de conexão ou corpos HTTP ausentes da origem.

## Consultar um PDF

No Workspace, pressione **u → a** para anexar o documento. A categoria **[18]** usa o primeiro PDF vinculado. **[18.2]** extrai texto, **[18.5]** mostra metadados e **[18.6]** lista imagens incorporadas. O texto extraído depende da camada textual do arquivo. Um PDF composto por páginas digitalizadas pode precisar de OCR em outra ferramenta.

## Editar comandos e tratar a saída

O editor completo da versão está em **[2.2]**, **[3.7]** e **[4.6]**. As demais ações expõem os ajustes que cada preset suporta. O preset original permanece disponível junto à prévia editável.

### Ajustar a seleção de pacotes

- **a → 1** e **a → 2** definem IP de origem e destino.
- **a → e** edita o filtro base `-Y`. Apague o campo para restaurar o original. **a → f** acrescenta um filtro.
- **a → E** edita os campos `-e`, separados por vírgula. Em **[2.2]** e **[4.6]**, mantenha os campos originais no início para as etapas que formatam endereços e payload.
- **a → p**, **r** e **u** controlam porta, direção e PSH nas ações TCP compatíveis.

Quando a seleção de pacotes ou os campos mudarem, execute novamente. A prévia acompanha a digitação e **Enter** confirma a edição.

### Organizar as linhas retornadas

Em modo guiado, **a → m** alterna ordem original, linhas únicas e contagem onde disponível. **a → g** agrupa por campo. No editor completo, você pode informar mais de um campo separado por vírgulas. **a → n** define um limite livre; **5/6/7** oferecem 10, 20 ou 50 linhas no editor. **a → d** decodifica payload quando disponível.

Pressione **t** para trabalhar com a sequência explícita. **a** adiciona, **e** edita, **Espaço** ativa ou desativa, **d** remove, **< / >** move e **r** restaura as etapas guiadas. As etapas usam a ordem escolhida. Por exemplo, posicione a ordenação antes de contar linhas repetidas com `uniq -c`.

| Entrada no editor de etapas | Efeito |
| --- | --- |
| `sort text asc` | Ordena linhas como texto. |
| `sort number desc 2` | Ordena numericamente pela segunda coluna separada por tab. |
| `sort version asc 1` | Usa ordenação natural na primeira coluna, útil para endereços IPv4 em formato pontuado. |
| `sort date asc 1` | Ordena datas ISO textualmente. Use valores no mesmo formato e fuso. |
| `uniq` / `uniq-count` | Remove ou conta linhas iguais consecutivas. |
| `head 10` / `tail 20` | Conserva as primeiras 10 ou últimas 20 linhas. |
| `grep literal exemplo` | Mantém linhas com o texto literal. Também há `ignore-case`, `regex` e `exclude`. |
| `cut 1,3` | Seleciona a primeira e a terceira coluna da saída tabulada. |
| `wc` | Conta as linhas. |
| `sum 1 2` / `mean 1 2` | Soma ou calcula a média da segunda coluna numérica por valor da primeira, usando uma receita de `awk`. |

A prévia copiável mostra os argumentos e as etapas reais. Use **c → c** para enviar ao clipboard se o terminal aceitar OSC 52; selecione manualmente o texto da mesma janela se necessário. **s** salva um conjunto de ajustes por ação no projeto, **b** carrega esse conjunto e **R** restaura o preset original no editor. Ao sair de uma ação com editor e alterações, escolha salvar ou descartar.

A TUI pode mostrar uma prévia de linhas brutas durante a execução. Ordenação e contagem são aplicadas à saída retida. Você pode ajustar as etapas depois e reaproveitar as linhas coletadas; o registro do histórico continua representando a execução gravada.

## Histórico, exportação e armazenamento

No Workspace, **y** abre o histórico e **f** abre favoritos. Abra uma execução anterior com **Enter**. Os **20** últimos resultados concluídos ficam no projeto.

| Opção em **x** | O que gera |
| --- | --- |
| **1** Texto | O texto do resultado. |
| **2** JSON | Metadados da ação e texto da saída, com indicação de corte quando aplicável. |
| **3** CSV | Uma coluna de número de linha e outra com o texto de cada linha. |
| **4** Markdown | Um relatório com o ID da ação e o resultado formatado como bloco de código. |
| **5** Formato nativo | Disponível quando a ação gera um formato nativo e a saída foi retida integralmente, sem limite de linhas. |

Ações como **[8.2] Exportar objetos HTTP**, **[19.8] Salvar pacotes por filtro** e **[19.19]** escrevem arquivos diretamente em uma nova pasta de exportação. O resultado na tela indica o caminho.

| Dado | Caminho padrão |
| --- | --- |
| Preferência de idioma | `~/.config/sonark/config.json` |
| Projeto | `~/.local/share/sonark/projects/saves/<projeto>/project.json` |
| Exportações | `~/.local/share/sonark/projects/saves/<projeto>/exports/<categoria>/` |

`XDG_CONFIG_HOME` e `XDG_DATA_HOME`, quando definidos com caminhos absolutos, substituem as bases padrão. Para fazer backup de uma investigação, copie a pasta do projeto e guarde as entradas e referências separadamente. Se os caminhos mudarem, selecione os arquivos na nova localização.

## Escopo e limites

| Tema | Comportamento |
| --- | --- |
| Aquisição | Leitura de arquivos locais. Captura ao vivo, replay de tráfego e varredura de rede ficam fora do escopo. |
| Formatos | PCAP/PCAPNG aceitos pelo TShark, logs locais de texto e ações específicas para PDFs anexados. Arquivos binários arbitrários não entram no fluxo de conversão de logs. |
| Cobertura | O editor completo está em três ações. Os demais presets têm ajustes próprios. Use **i** para os IDs pendentes e consulte o [catálogo](catalogo.md) para os implementados. |
| Evidência | Uma representação derivada de log descreve eventos registrados. A visibilidade de campos em capturas depende dos pacotes retidos, da cifragem e do dissector. |
| Volume | O leitor de logs tem prévia de até **16 MiB**, a conversão aceita até **64 MiB** de texto e a análise retém até **2 MiB** de saída. Agregações de grandes saídas podem refletir somente as linhas retidas. |
| Armazenamento | Notas, históricos e relatórios são arquivos locais legíveis. A aplicação não faz mascaramento automático de dados da captura. |
| Concorrência | A TUI permite navegar com uma execução em segundo plano e processa uma execução externa por vez. |

## Resolver problemas

| Sintoma | O que conferir |
| --- | --- |
| O projeto não compila offline | Na primeira compilação, rode `cargo build --locked` para preparar o cache do Cargo. |
| A TUI não abre ou não responde como esperado | Rode no terminal, com entrada e saída interativas e sem redirecionamento. Use **F1** para o guia e **?** para atalhos. |
| O executor informa que falta um programa | Confira os comandos no `PATH`. O log direto, as conversões, a análise PCAP e os PDFs usam dependências diferentes. |
| O TShark rejeita um campo ou filtro | Verifique a grafia dos campos e a compatibilidade com sua versão do TShark. No editor, compare a prévia com o original. |
| A ação pede captura mesmo com um log anexado | Escolha uma ação **[LOG]** ou ative uma captura apropriada em **p**. A conversão é útil apenas para campos representados no PCAPNG derivado. |
| O parser HTTP não encontrou registros | Confira se o texto segue Common ou Combined. Para outro formato, use o leitor de texto, a conversão genérica ou, quando aplicável, o modo JSON. |
| O resultado aparece cortado | A saída atingiu o limite de retenção. Reduza a entrada com um filtro de exibição, ajuste a janela de tempo ou extraia somente os campos relevantes. Limitar as linhas exibidas não recupera dados cortados. |
| A opção **5** não aparece | Ela depende de formato nativo e resultado completo. Remova o limite de linhas e execute novamente com uma saída menor. |
| O arquivo vinculado foi movido | Abra **p** e selecione o caminho novo. |
| O comando não foi para a área de transferência | Confira se o terminal aceita OSC 52. A janela **c** também permite copiar o texto manualmente. |

[Voltar ao README](../README.md)
