# Skills Frontend & UI/UX

> Design et développement d'interfaces utilisateur de qualité production.

---

## Vue d'ensemble

| Skill | Usage |
|-------|-------|
| `frontend-design` | Interfaces production-grade, anti-AI-slop |
| `shadcn-ui` | Composants React accessibles + Tailwind |
| `web-design-guidelines` | Review UI selon guidelines |
| `remotion-best-practices` | Création vidéo en React |
| `vercel-react-best-practices` | Performance React/Next.js |

---

## 1. frontend-design

**Source:** `~/.agents/skills/frontend-design/`

### Quand l'utiliser

- Construire des composants web, pages, dashboards
- Créer des landing pages
- Styliser/embellir une UI
- Éviter l'esthétique "AI générique"

### Philosophie

> Créer des interfaces **distinctives** et **production-grade** qui évitent l'esthétique "AI slop".

### Design Thinking (avant de coder)

1. **Purpose**: Quel problème cette interface résout?
2. **Tone**: Choisir un extrême:
   - Brutally minimal
   - Maximalist chaos
   - Retro-futuristic
   - Organic/natural
   - Luxury/refined
   - Playful/toy-like
   - Editorial/magazine
   - Brutalist/raw
   - Art deco/geometric
   - Soft/pastel
   - Industrial/utilitarian

3. **Contraintes**: Framework, performance, accessibilité
4. **Différenciation**: Qu'est-ce qui rend ça INOUBLIABLE?

### Guidelines esthétiques

#### Typography
- Polices **belles, uniques, intéressantes**
- **ÉVITER**: Arial, Inter, Roboto, system fonts
- Pairer une display font distinctive avec une body font raffinée

#### Color & Theme
- Palette cohésive avec variables CSS
- Couleurs dominantes avec accents marqués
- Éviter les palettes timides et également distribuées

#### Motion
- Animations pour effets et micro-interactions
- Priorité aux solutions CSS-only pour HTML
- Motion library pour React
- Un page load bien orchestré > micro-interactions dispersées

#### Spatial Composition
- Layouts inattendus
- Asymétrie, overlap, flux diagonal
- Éléments qui brisent la grille
- Espace négatif généreux OU densité contrôlée

#### Backgrounds & Visual Details
- Créer atmosphère et profondeur
- Gradient meshes, noise textures
- Patterns géométriques
- Transparences layered
- Shadows dramatiques
- Grain overlays

### CE QU'IL NE FAUT JAMAIS FAIRE

❌ Font families overused (Inter, Roboto, Arial)
❌ Color schemes clichés (purple gradients on white)
❌ Layouts prévisibles
❌ Cookie-cutter design sans caractère

---

## 2. shadcn-ui

**Source:** `~/.agents/skills/shadcn-ui/`

### Qu'est-ce que shadcn/ui?

Collection de composants React réutilisables:
- **Pas un package npm** - Tu copies le code dans ton projet
- **Tu possèdes le code** - Personnalisation totale
- **Radix UI** pour l'accessibilité
- **Tailwind CSS** pour le styling
- **CLI tool** pour installation facile

### Quick Start

```bash
# Nouveau projet
npx create-next-app@latest my-app --typescript --tailwind --eslint --app
cd my-app
npx shadcn@latest init

# Installer composants
npx shadcn@latest add button input form card dialog select
```

### Installation composants

```bash
# Un composant
npx shadcn@latest add button

# Plusieurs
npx shadcn@latest add button input form

# Tous
npx shadcn@latest add --all
```

### Composants principaux

#### Button
```tsx
import { Button } from "@/components/ui/button";

// Variants
<Button variant="default">Default</Button>
<Button variant="destructive">Destructive</Button>
<Button variant="outline">Outline</Button>
<Button variant="ghost">Ghost</Button>

// Sizes
<Button size="sm">Small</Button>
<Button size="lg">Large</Button>
<Button size="icon"><Icon /></Button>
```

#### Form avec validation (Zod + React Hook Form)
```tsx
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import * as z from "zod";

const formSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
});

function LoginForm() {
  const form = useForm({
    resolver: zodResolver(formSchema),
  });

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit">Login</Button>
      </form>
    </Form>
  );
}
```

#### Dialog (Modal)
```tsx
<Dialog>
  <DialogTrigger asChild>
    <Button>Open</Button>
  </DialogTrigger>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Title</DialogTitle>
      <DialogDescription>Description</DialogDescription>
    </DialogHeader>
    {/* Content */}
    <DialogFooter>
      <Button>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
```

#### Select (Dropdown)
```tsx
<Select>
  <SelectTrigger>
    <SelectValue placeholder="Select..." />
  </SelectTrigger>
  <SelectContent>
    <SelectItem value="a">Option A</SelectItem>
    <SelectItem value="b">Option B</SelectItem>
  </SelectContent>
</Select>
```

#### Toast
```tsx
// Setup dans layout
<Toaster />

// Usage
const { toast } = useToast();
toast({
  title: "Success",
  description: "Changes saved.",
});

// Error
toast({
  variant: "destructive",
  title: "Error",
  description: "Something went wrong.",
});
```

### Theming avec CSS Variables

```css
/* globals.css */
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  --primary: 222.2 47.4% 11.2%;
  --primary-foreground: 210 40% 98%;
  /* ... */
}

.dark {
  --background: 222.2 84% 4.9%;
  --foreground: 210 40% 98%;
  /* ... */
}
```

---

## 3. web-design-guidelines

**Source:** `~/.agents/skills/web-design-guidelines/`

### Quand l'utiliser

- Review UI
- Check accessibility
- Audit design
- Review UX
- Check site against best practices

### Ce que ça fait

Review le code UI selon les Web Interface Guidelines pour:
- Accessibilité
- Usabilité
- Responsive design
- Performance perçue
- Best practices

---

## 4. remotion-best-practices

**Source:** `~/.agents/skills/remotion-best-practices/`

### Qu'est-ce que Remotion?

Création de vidéos programmatiques en React.

### Topics couverts

- **3D**: Intégration Three.js
- **Animations**: useCurrentFrame, interpolate
- **Assets**: Gestion médias
- **Audio**: Intégration audio
- **Charts**: Graphiques animés
- **Compositions**: Structure des vidéos
- **Captions**: Sous-titres
- **Fonts**: Custom fonts
- **GIFs**: Intégration GIFs
- **Transitions**: Entre séquences
- **Tailwind**: Styling avec Tailwind

### Exemple basique

```tsx
import { useCurrentFrame, interpolate } from 'remotion';

export const MyVideo = () => {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 30], [0, 1]);

  return (
    <div style={{ opacity }}>
      Hello World
    </div>
  );
};
```

---

## 5. vercel-react-best-practices

**Source:** `~/.agents/skills/vercel-react-best-practices/`

### Quand l'utiliser

- Écrire/review code React/Next.js
- Optimisation performance
- Data fetching
- Bundle optimization

### Catégories de règles

#### Rendering
- Conditional rendering
- Hydration (no flicker)
- SVG precision
- useTransition pour loading

#### Re-renders
- useMemo correctement
- Derived state (pas d'effect)
- Functional setState
- Lazy state initialization

#### Async
- Parallel fetching
- Suspense boundaries
- API routes

#### Bundle
- Barrel imports (éviter)
- Dynamic imports
- Defer third-party
- Preload

#### Server (Next.js)
- Server cache (LRU)
- React cache
- Parallel fetching
- Serialization

#### JavaScript
- Batch DOM/CSS operations
- Cache function results
- Early exit
- Set/Map lookups

### Exemple: Éviter barrel imports

```typescript
// ❌ Mauvais - importe tout le barrel
import { Button, Input, Card } from '@/components/ui';

// ✅ Bon - imports directs
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
```

### Exemple: Derived state sans effect

```typescript
// ❌ Mauvais - effet inutile
const [items, setItems] = useState([]);
const [count, setCount] = useState(0);

useEffect(() => {
  setCount(items.length);
}, [items]);

// ✅ Bon - calcul direct
const [items, setItems] = useState([]);
const count = items.length;
```

---

## Références

| Skill | Documentation officielle |
|-------|-------------------------|
| shadcn/ui | https://ui.shadcn.com |
| Radix UI | https://radix-ui.com |
| Tailwind | https://tailwindcss.com |
| Remotion | https://remotion.dev |
| React Hook Form | https://react-hook-form.com |
| Zod | https://zod.dev |

---

*Dernière mise à jour: 2026-01-27*
