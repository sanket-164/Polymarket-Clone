export function getStatusColor(status: string) {
    switch (status) {
        case "ACTIVE":
            return "bg-emerald-500/10 text-emerald-400 ring-1 ring-emerald-500/20";
        case "CLOSED":
            return "bg-slate-500/10 text-slate-400 ring-1 ring-slate-500/20";
        case "RESOLVED":
            return "bg-blue-500/10 text-blue-400 ring-1 ring-blue-500/20";
        case "PENDING":
            return "bg-amber-500/10 text-amber-400 ring-1 ring-amber-500/20";
        default:
            return "bg-slate-500/10 text-slate-400 ring-1 ring-slate-500/20";
    }
}

export function formatDateTime(value: string) {
    const date = new Date(value)

    if (Number.isNaN(date.getTime())) {
        return 'Unknown date'
    }

    return new Intl.DateTimeFormat('en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
    }).format(date)
}
