import os
import sys
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
from pathlib import Path
import threading

# --- CONFIGURAÇÕES ADAPTADAS PARA ANIMEHUB (TAURI + SVELTEKIT) ---
OUTPUT_FILENAME = "contexto_animehub.txt"
MAX_FILE_SIZE_KB = 250  # Aumentado levemente para arquivos de regras de domínio complexas

IGNORE_DIRS = {
    '.git', '.idea', '.vscode', '.venv', '__pycache__', 'venv', 'env',
    'node_modules', '.next', 'out', 'dist', 'build', 'target', # 'target' é essencial para Rust
    'public', 'static', 'storage', 'logs', 'assets', 'images', 'img', 
    'cov', 'coverage', '.backup_temp', '.github', 'fixtures', 'e2e',
    'prisma/migrations', 'bin'
}

IGNORE_FILES = {
    'pnpm-lock.yaml', 'package-lock.json', 'yarn.lock', 'bun.lockb', 'Cargo.lock',
    'projeto_completo.txt', 'contexto_animehub.txt', 'pack.py', '.DS_Store',
    'favicon.ico', 'next-env.d.ts', '.gitignore', 'LICENSE',
    '.env.local', '.env.example', 'requirements.txt',
    'tsconfig.json', 'jsconfig.json', 'package-lock.json',
    'next.config.js', 'jest.config.js', 'playwright.config.js', 'postcss.config.js',
    '.eslintrc.json', '.prettierrc', '.browserslistrc', '.flake8',
    'tailwind.config.ts', 'check-deps.js', 'install.js', 'start.js'
}

# Extensões cruciais para o ecossistema Rust/Svelte
INCLUDE_EXTS = {
    '.rs', '.svelte', '.ts', '.tsx', '.js', '.jsx',
    '.toml', '.json', '.sql', '.prisma',
    '.md', '.txt', '.css', '.html', '.sh'
}

class ProjectPackerApp:
    def __init__(self, root):
        self.root = root
        self.root.title("AnimeHub - LLM Context Packer")
        self.root.geometry("1100x850") # Corrigido de 11000 para 1100
        
        self.tree_items = {}  # item_id -> Path
        self.check_states = {} # item_id -> bool
        
        self._setup_ui()
        self._start_scan()

    def _setup_ui(self):
        top_frame = ttk.Frame(self.root, padding=10)
        top_frame.pack(fill=tk.X)
        
        ttk.Label(top_frame, text="Raíz do Projeto:").pack(side=tk.LEFT)
        self.path_var = tk.StringVar(value=os.getcwd())
        self.entry_path = ttk.Entry(top_frame, textvariable=self.path_var, width=60)
        self.entry_path.pack(side=tk.LEFT, padx=5)
        
        ttk.Button(top_frame, text="📁 Selecionar Pasta", command=self._browse_folder).pack(side=tk.LEFT, padx=2)
        ttk.Button(top_frame, text="🔄 Rescanear", command=self._start_scan).pack(side=tk.LEFT, padx=2)

        tree_frame = ttk.Frame(self.root, padding=10)
        tree_frame.pack(fill=tk.BOTH, expand=True)

        vsb = ttk.Scrollbar(tree_frame, orient="vertical")
        hsb = ttk.Scrollbar(tree_frame, orient="horizontal")
        
        self.tree = ttk.Treeview(
            tree_frame, 
            columns=("size", "status"), 
            selectmode="browse", 
            yscrollcommand=vsb.set,
            xscrollcommand=hsb.set
        )
        
        self.tree.heading("#0", text="Estrutura de Arquivos (AnimeHub)", anchor=tk.W)
        self.tree.heading("size", text="Tamanho", anchor=tk.E)
        self.tree.heading("status", text="Tipo", anchor=tk.CENTER)
        
        self.tree.column("#0", minwidth=500, width=600)
        self.tree.column("size", width=100, anchor=tk.E)
        self.tree.column("status", width=100, anchor=tk.CENTER)
        
        vsb.config(command=self.tree.yview)
        hsb.config(command=self.tree.xview)
        
        self.tree.pack(side=tk.TOP, fill=tk.BOTH, expand=True)
        vsb.pack(side=tk.RIGHT, fill=tk.Y)
        hsb.pack(side=tk.BOTTOM, fill=tk.X)

        self.tree.bind("<Button-1>", self._on_tree_click)
        self.tree.bind("<space>", self._on_tree_space)

        btm_frame = ttk.Frame(self.root, padding=10)
        btm_frame.pack(fill=tk.X)

        self.lbl_stats = ttk.Label(btm_frame, text="Aguardando...", font=("Consolas", 10))
        self.lbl_stats.pack(side=tk.LEFT)

        btn_frame = ttk.Frame(btm_frame)
        btn_frame.pack(side=tk.RIGHT)

        ttk.Button(btn_frame, text="⚡ Auto Selecionar Válidos", command=self.auto_select_valid).pack(side=tk.LEFT, padx=5)
        ttk.Separator(btn_frame, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=5)
        ttk.Button(btn_frame, text="📋 Copiar para LLM", command=self.copy_to_clipboard).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="💾 Salvar .txt", command=self.save_file_as).pack(side=tk.LEFT, padx=5)

    def _browse_folder(self):
        path = filedialog.askdirectory(initialdir=self.path_var.get())
        if path:
            self.path_var.set(path)
            self._start_scan()

    def _is_valid_file(self, full_path):
        if full_path.name in IGNORE_FILES:
            return False
        if full_path.suffix.lower() not in INCLUDE_EXTS:
            return False
        try:
            if full_path.stat().st_size > MAX_FILE_SIZE_KB * 1024:
                return False
        except OSError:
            return False
        return True

    def _start_scan(self):
        self.tree.delete(*self.tree.get_children())
        self.tree_items.clear()
        self.check_states.clear()
        
        root_path = Path(self.path_var.get())
        if not root_path.exists():
            messagebox.showerror("Erro", "Caminho raiz não encontrado.")
            return

        self.lbl_stats.config(text="Escaneando diretórios...")
        threading.Thread(target=self._run_scan, args=(root_path,), daemon=True).start()

    def _run_scan(self, root_path):
        self._scan_and_insert_ordered(root_path, "")
        self.root.after(0, lambda: self.lbl_stats.config(text="Escaneamento concluído."))
        self.root.after(0, self._update_stats)

    def _scan_and_insert_ordered(self, current_path, parent_id):
        try:
            # Prioriza docs na árvore visual também
            entries = sorted(os.listdir(current_path), key=lambda x: (
                not os.path.isdir(current_path / x), 
                x.lower() != 'docs', 
                x.lower()
            ))
        except PermissionError:
            return

        for entry in entries:
            if entry in IGNORE_DIRS: continue
            
            full_path = current_path / entry
            is_dir = full_path.is_dir()
            
            item_id = self._sync_insert(parent_id, entry, full_path, is_dir)
            
            if is_dir:
                self._scan_and_insert_ordered(full_path, item_id)

    def _sync_insert(self, parent_id, text, full_path, is_dir):
        res = {"id": None}
        event = threading.Event()

        def do_insert():
            size_str = ""
            if not is_dir:
                try: size_str = f"{full_path.stat().st_size / 1024:.1f} KB"
                except: size_str = "Error"
            
            icon = "📁" if is_dir else "📄"
            item_id = self.tree.insert(parent_id, "end", text=f" ⬜ {icon} {text}", values=(size_str, "Pasta" if is_dir else "Arquivo"))
            self.tree_items[item_id] = full_path
            self.check_states[item_id] = False
            res["id"] = item_id
            event.set()

        self.root.after(0, do_insert)
        event.wait()
        return res["id"]

    def _update_visual_check(self, item_id):
        state = self.check_states.get(item_id, False)
        path = self.tree_items[item_id]
        icon = "📁" if path.is_dir() else "📄"
        mark = "✅" if state else "⬜"
        
        text = self.tree.item(item_id, "text")
        clean_text = text.split(f" {icon} ")[-1]
        
        self.tree.item(item_id, text=f" {mark} {icon} {clean_text}")

    def _toggle_check_recursive(self, item_id, force_state=None):
        current_state = self.check_states.get(item_id, False)
        new_state = not current_state if force_state is None else force_state
        
        self.check_states[item_id] = new_state
        self._update_visual_check(item_id)
        
        for child in self.tree.get_children(item_id):
            self._toggle_check_recursive(child, new_state)

    def _on_tree_click(self, event):
        region = self.tree.identify("region", event.x, event.y)
        if region == "tree":
            item_id = self.tree.identify_row(event.y)
            if item_id:
                self._toggle_check_recursive(item_id)
                self._update_stats()

    def _on_tree_space(self, event):
        item_id = self.tree.focus()
        if item_id:
            self._toggle_check_recursive(item_id)
            self._update_stats()

    def auto_select_valid(self):
        for item_id, path in self.tree_items.items():
            if path.is_file() and self._is_valid_file(path):
                self.check_states[item_id] = True
                self._update_visual_check(item_id)
        self._update_stats()

    def _update_stats(self):
        files = [p for i, p in self.tree_items.items() if self.check_states[i] and p.is_file()]
        total_kb = sum(p.stat().st_size for p in files if p.exists()) / 1024
        self.lbl_stats.config(text=f"Selecionados: {len(files)} arquivos | Tamanho Total: {total_kb:.1f} KB")

    def _generate_ascii_tree(self):
        lines = ["."]
        def walk_tree(item_id, prefix=""):
            children = self.tree.get_children(item_id)
            for i, child_id in enumerate(children):
                path = self.tree_items[child_id]
                is_last = (i == len(children) - 1)
                connector = "└── " if is_last else "├── "
                
                state_mark = "[X]" if self.check_states[child_id] else "[ ]"
                if path.is_dir():
                    state_mark = "DIR"
                
                lines.append(f"{prefix}{connector}{state_mark} {path.name}")
                if path.is_dir():
                    new_prefix = prefix + ("    " if is_last else "│   ")
                    walk_tree(child_id, new_prefix)
        walk_tree("")
        return "\n".join(lines)

    def _generate_final_content(self):
        root_dir = Path(self.path_var.get())
        selected_files = [
            path for i_id, path in self.tree_items.items() 
            if self.check_states[i_id] and path.is_file()
        ]

        # Lógica de Ordenação: DOCUMENTOS PRIMEIRO
        def sort_key(p):
            rel = p.relative_to(root_dir)
            parts = [part.lower() for part in rel.parts]
            
            # 1. Prioridade máxima para a pasta 'docs'
            is_in_docs = 'docs' in parts
            # 2. Prioridade para arquivos de documentação na raiz (README, ARCHITECTURE, etc)
            is_root_doc = len(parts) == 1 and rel.suffix.lower() in ['.md', '.txt']
            
            # O Python ordena False antes de True, então usamos 0 para docs e 1 para o resto
            priority = 0 if (is_in_docs or is_root_doc) else 1
            return (priority, str(rel).lower())

        sorted_files = sorted(selected_files, key=sort_key)

        output = [
            f"=== PROJETO: {root_dir.name} ===\n",
            "=== ESTRUTURA VISUAL ===\n",
            self._generate_ascii_tree(),
            f"\n\n{'='*60}\n",
            "CONTEÚDO DOS ARQUIVOS (Ordenado: Docs -> Source)\n",
            f"{'='*60}\n"
        ]

        for path in sorted_files:
            rel_path = path.relative_to(root_dir)
            output.append(f"\n--- FILE: {rel_path} ---\n")
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    output.append(f.read())
                output.append("\n")
            except Exception as e:
                output.append(f"[ERRO AO LER ARQUIVO: {e}]\n")

        return "".join(output)

    def copy_to_clipboard(self):
        content = self._generate_final_content()
        self.root.clipboard_clear()
        self.root.clipboard_append(content)
        messagebox.showinfo("Sucesso", "Contexto do AnimeHub copiado!")

    def save_file_as(self):
        content = self._generate_final_content()
        target = filedialog.asksaveasfilename(
            defaultextension=".txt", 
            initialfile=OUTPUT_FILENAME,
            title="Salvar Contexto do Projeto"
        )
        if target:
            with open(target, 'w', encoding='utf-8') as f:
                f.write(content)
            messagebox.showinfo("Sucesso", f"Salvo em:\n{target}")

if __name__ == "__main__":
    root = tk.Tk()
    style = ttk.Style()
    style.theme_use('clam')
    style.configure("Treeview", rowheight=25, font=('Consolas', 10))
    style.configure("Treeview.Heading", font=('Segoe UI', 10, 'bold'))
    
    app = ProjectPackerApp(root)
    root.mainloop()