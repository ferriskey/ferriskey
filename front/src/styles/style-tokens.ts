/**
 * Les jetons du style style FerrisKey — l'unique style de la console IAM.
 *
 * Le kit d'exploration (`ferriskey-kit`) portait quatre styles et un
 * `StyleProvider` pour basculer entre eux. Un seul a été acté : il n'y a donc
 * plus rien à choisir à l'exécution, et les jetons se lisent par un import
 * constant plutôt que par un contexte React. On garde la forme d'objet — un
 * écran qui a besoin du padding de cellule le lit ici au lieu de le recopier
 * en dur, ce qui est tout l'intérêt de l'exercice.
 *
 * Les valeurs viennent de `styleTokens.ferriskey` du kit et sont consignées
 * dans le référentiel FK-01 … FK-43 (section « Surfaces & densité »).
 *
 * Le style FerrisKey est la synthèse de trois styles essayés avant lui :
 *  - la densité du compact      (padding serré, 13 px, arrondis minimaux)
 *  - la lecture du data-first   (colonnes secondaires, tri, agrégats)
 *  - les alertes de l'opérationnel (bandeau d'anomalies, pastilles d'état)
 *
 * Ce qu'il n'emprunte pas compte autant : le compact masque les descriptions
 * de page, le style FerrisKey les garde. Il prend la densité, pas le dépouillement.
 */
export interface StyleTokens {
  /* Mise en page ---------------------------------------------------- */
  page: {
    padding: string
    maxWidth: string
    /** Écart entre les blocs de chrome d'un listing (métriques, barre, table). */
    sectionGap: string
    /**
     * Écart entre deux <Section> encartées d'une page de détail. Plus large
     * que sectionGap : deux panneaux collés se lisent comme un seul bloc.
     */
    blockGap: string
  }

  /* En-tête de page ------------------------------------------------- */
  header: {
    title: string
    spacing: string
  }

  /* Tables ---------------------------------------------------------- */
  table: {
    cellPadding: string
    headerPadding: string
    text: string
    headerText: string
    showFooterAggregates: boolean
    sortable: boolean
    selectable: boolean
  }

  /* Cartes ---------------------------------------------------------- */
  card: {
    padding: string
    gap: string
    columns: string
    showFlows: boolean
  }

  /* Surfaces -------------------------------------------------------- */
  surface: {
    /** Conteneur de section standard. */
    panel: string
    /** Séparateur entre lignes. */
    divider: string
  }

  /* Barre d'outils -------------------------------------------------- */
  toolbar: {
    showQuerySyntax: boolean
    showViewToggle: boolean
  }
}

export const tokens: StyleTokens = {
  page: {
    padding: 'px-5 py-4',
    maxWidth: 'max-w-[1600px]',
    sectionGap: 'space-y-3',
    blockGap: 'space-y-6',
  },
  header: {
    title: 'text-lg font-semibold tracking-tight',
    spacing: 'pb-3',
  },
  table: {
    cellPadding: 'px-2.5 py-2',
    headerPadding: 'px-2.5 py-1.5',
    text: 'text-[13px]',
    headerText: 'text-[11px] font-medium uppercase tracking-wide text-neutral-400',
    // FK-03 : pas de zébrage — les lignes alternées entreraient en
    // concurrence avec les fonds d'état (-soft), qui portent une information.
    showFooterAggregates: true,
    sortable: true,
    selectable: true,
  },
  card: {
    padding: 'p-3',
    gap: 'gap-2',
    columns: 'sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4',
    showFlows: true,
  },
  surface: {
    panel: 'rounded-sm border border-fk-line bg-white',
    divider: 'divide-y divide-fk-line-soft',
  },
  toolbar: {
    showQuerySyntax: true,
    showViewToggle: true,
  },
}
