<script lang="ts">
    // Links to the main admin sections
    const adminLinks = [
        { title: "Gerir Páginas", href: "/admin/pages/", icon: "file-lines" },
        { title: "Gerir Utilizadores", href: "/admin/users/", icon: "users" },
        { title: "Gerir Funções", href: "/admin/roles/", icon: "user-tag" },
        // Add more admin links here if needed
    ];
</script>

<div class="card bg-base-100 shadow-xl">
    <div class="card-body">
        <h2 class="card-title text-primary">
            <i class="fa-solid fa-user-shield mr-2"></i>
            Painel de Administração
        </h2>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 mt-2">
            {#each adminLinks as link}
                <a
                    href={link.href}
                    class="btn btn-outline flex items-center justify-start gap-2 hover:bg-primary hover:text-primary-content"
                >
                    <i class="fa-solid fa-{link.icon} w-4 text-center"></i>
                    <span class="truncate">{link.title}</span>
                </a>
            {/each}
        </div>
    </div>
</div>
