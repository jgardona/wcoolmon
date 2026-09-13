# Otimizar tamanho do binário e melhorar o código do wcoolmon

## Contexto

`wcoolmon` é um utilitário de linha de comando de arquivo único (`src/main.rs`, 67 linhas) que lê a
temperatura da CPU via `sysinfo` e envia o valor para um dispositivo HID (water cooler). O usuário
quer (1) reduzir o tamanho do executável final e (2) melhorar a qualidade do código.

O `Cargo.toml` já tem um `[profile.release]` decente (`strip = true`, `opt-level = "z"`, `lto = true`,
`panic = "abort"`), então o maior ganho de tamanho restante vem de **cortar features não usadas da
dependência `sysinfo`**, que por padrão compila suporte para disco, rede, sistema e usuários — dos
quais o programa só usa `Components` (sensores/temperatura). O usuário confirmou manter `clap` como
está (boa UX de `--help`/`--version`/validação), então não vamos trocar o parser de CLI.

Na revisão do código (`src/main.rs`), foi encontrado um bug real: quando a escrita no dispositivo HID
falha, o loop principal só imprime o erro e faz `break`, mas a função `main` ainda retorna `Ok(())`,
ou seja, o processo sai com código de saída 0 mesmo em falha. Isso é relevante porque o projeto tem um
template de serviço systemd (`Restart=on-failure` normalmente depende do exit code) — um exit code 0
falso pode impedir o systemd de reiniciar o serviço quando o dispositivo cai. Também há uma forma um
pouco obscura de extrair a temperatura (`.map(...)?` dentro de uma função que retorna `Option`) que
pode ser simplificada com `.and_then(...)`.

## Mudanças

### 1. `Cargo.toml` — reduzir dependências compiladas

- Trocar a linha `sysinfo = "0.38.4"` por:
  ```toml
  sysinfo = { version = "0.38.4", default-features = false, features = ["component"] }
  ```
  Isso remove o código de disco/rede/sistema/usuários que nunca é usado (só `Components` é
  referenciado em [main.rs](src/main.rs)).
- Adicionar `codegen-units = 1` ao `[profile.release]` (ajuda o LTO a otimizar/enxugar mais,
  complementando `lto = true` e `opt-level = "z"` que já existem).

### 2. `src/main.rs` — correção de bug + simplificação

- **Bug de exit code**: no `match device.write(&command)`, o braço `Err(e)` hoje faz
  `eprintln!(...); break;` e a função termina em `Ok(())`. Trocar para propagar o erro (ex.: usar
  `anyhow::bail!` ou retornar o `Err` convertido) para que `main` retorne `Err` e o processo saia com
  código de saída diferente de zero quando o dispositivo falha — assim o systemd
  (`Restart=on-failure`) consegue reiniciar o serviço corretamente.
- **Simplificar `get_cpu_temp`**: trocar o `.map(|c| c.temperature())?` (que depende do operador `?`
  em `Option` dentro de uma função que retorna `Option<f32>`, forma pouco óbvia) por
  `.and_then(|c| c.temperature())`, que expressa a mesma lógica de forma direta e idiomática.
- **Evitar alocação eager no caminho de sucesso**: trocar
  `.context(format!("Device {:04x}:{:04x} not found", ...))` por
  `.with_context(|| format!(...))`, para que a string só seja formatada quando o `open` realmente
  falha (hoje é formatada sempre, mesmo em caso de sucesso).

Nenhuma outra mudança de comportamento (CLI, argumentos, protocolo HID) é necessária.

## Verificação

1. `cargo build --release` e comparar o tamanho de `target/release/wcoolmon` antes e depois das
   mudanças (`ls -la` / `du -h`).
2. `cargo clippy --release` para garantir que nada quebrou e que a simplificação do `get_cpu_temp`
   não introduz warnings.
3. `cargo run --release -- --help` para confirmar que a CLI (clap) continua funcionando normalmente.
4. Testar manualmente com o dispositivo HID conectado (se disponível no ambiente), incluindo o caso de
   desconexão do dispositivo em execução, para confirmar que o processo agora sai com código de saída
   diferente de zero (`echo $?` após o processo terminar por erro de escrita).