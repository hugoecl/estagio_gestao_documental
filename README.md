# Projeto de estágio - Desenvolvimento de uma plataforma para Gestão Documental

Projeto de estágio realizado na empresa [**JCC**](https://www.jcc.pt/), com o objetivo de desenvolver uma plataforma para gestão documental.

## Tecnologias utilizadas no projeto

- **Frontend**:
  - [Svelte](https://svelte.dev/)
  - [Astro](https://astro.build/)
  - [Tailwind CSS](https://tailwindcss.com/)
  - [DaisyUI](https://daisyui.com/)
  - [Bun](https://bun.sh/)
- **Backend**:
  - [Rust](https://www.rust-lang.org/)
  - [Actix Web](https://actix.rs/)
  - [SQLx](https://github.com/launchbadge/sqlx)
  - [MySQL](https://www.mysql.com/)

## Executar o projeto

### Frontend

```bash
cd frontend
bun install

# dev
bun run dev

# release
bun run build
./publish.sh # Script to update nginx root dir
```

### Backend

```bash
cd backend
# dev
cargo run -- --help

# release
cargo run --release --help
```
