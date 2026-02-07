import { useEffect, useState } from "react";

interface Props {
  step: number;
  onNext: () => void;
  onSkip: () => void;
}

const steps = [
  {
    title: "Bienvenue sur ClipFlow !",
    description: "Enregistrez plusieurs zones de votre écran et assemblez-les automatiquement avec des transitions fluides.",
    target: null,
  },
  {
    title: "1. Sélectionner une zone",
    description: "Cliquez sur \"Nouvelle Zone\" pour dessiner un rectangle sur votre écran, ou cliquez sur une fenêtre pour la capturer.",
    target: "data-onboarding-region",
  },
  {
    title: "2. Enregistrer",
    description: "Appuyez sur F9 ou cliquez Record pour démarrer. F10 pour pause, F9 à nouveau pour arrêter.",
    target: "data-onboarding-record",
  },
  {
    title: "3. Timeline et transitions",
    description: "Vos clips apparaissent ici. Réordonnez-les par glisser-déposer et choisissez des transitions entre chaque clip.",
    target: "data-onboarding-timeline",
  },
  {
    title: "4. Exporter",
    description: "Exportez en MP4 ou GIF avec la qualité de votre choix. Le fichier s'ouvre automatiquement dans votre dossier.",
    target: "data-onboarding-export",
  },
];

const CARD_WIDTH = 340;
const CARD_HEIGHT_ESTIMATE = 180;
const GAP = 16;

export function OnboardingOverlay({ step, onNext, onSkip }: Props) {
  const currentStep = steps[step];
  const [, forceUpdate] = useState(0);

  // Re-render on resize so positions stay accurate
  useEffect(() => {
    const handler = () => forceUpdate((n) => n + 1);
    window.addEventListener("resize", handler);
    return () => window.removeEventListener("resize", handler);
  }, []);

  if (!currentStep) return null;

  const isLastStep = step === steps.length - 1;

  // Find target element
  let targetRect: DOMRect | null = null;
  if (currentStep.target) {
    const el = document.querySelector(`[${currentStep.target}]`);
    if (el) {
      targetRect = el.getBoundingClientRect();
    }
  }

  // Compute card position
  let cardStyle: React.CSSProperties;
  const isLargeTarget = targetRect && (
    targetRect.height > window.innerHeight * 0.5 ||
    targetRect.width > window.innerWidth * 0.7
  );

  if (targetRect && !isLargeTarget) {
    // Small target: place card below or above
    const spaceBelow = window.innerHeight - targetRect.bottom - GAP;
    const spaceAbove = targetRect.top - GAP;
    const placeAbove = spaceBelow < CARD_HEIGHT_ESTIMATE && spaceAbove > spaceBelow;

    const top = placeAbove
      ? Math.max(GAP, targetRect.top - CARD_HEIGHT_ESTIMATE - GAP)
      : targetRect.bottom + GAP;

    const left = Math.max(GAP, Math.min(targetRect.left, window.innerWidth - CARD_WIDTH - GAP));

    cardStyle = { left, top };
  } else {
    // No target or large target: center the card
    cardStyle = { left: "50%", top: "50%", transform: "translate(-50%, -50%)" };
  }

  // Spotlight rect with padding
  const spot = targetRect
    ? {
        left: targetRect.left - 10,
        top: targetRect.top - 10,
        width: targetRect.width + 20,
        height: targetRect.height + 20,
      }
    : null;

  return (
    <div className="fixed inset-0 z-[100]">
      {/* Single dark overlay — uses spotlight box-shadow when there's a target */}
      {spot ? (
        <div
          className="absolute rounded-xl border-2 border-blue-400/80"
          style={{
            left: spot.left,
            top: spot.top,
            width: spot.width,
            height: spot.height,
            boxShadow: "0 0 0 9999px rgba(0, 0, 0, 0.65)",
            zIndex: 101,
          }}
        />
      ) : (
        <div className="absolute inset-0 bg-black/65" />
      )}

      {/* Content card */}
      <div className="absolute z-[102]" style={cardStyle}>
        <div className="bg-white dark:bg-zinc-900 rounded-2xl shadow-2xl p-5 w-[340px] border border-zinc-200 dark:border-zinc-700">
          {/* Step dots */}
          <div className="flex items-center gap-1.5 mb-3">
            {steps.map((_, i) => (
              <div
                key={i}
                className={`h-1.5 rounded-full transition-all ${
                  i === step
                    ? "w-6 bg-blue-500"
                    : i < step
                    ? "w-3 bg-blue-400/60"
                    : "w-3 bg-zinc-300 dark:bg-zinc-700"
                }`}
              />
            ))}
          </div>

          <h3 className="text-base font-semibold text-zinc-900 dark:text-white mb-2">
            {currentStep.title}
          </h3>
          <p className="text-sm text-zinc-600 dark:text-zinc-300 leading-relaxed mb-5">
            {currentStep.description}
          </p>

          <div className="flex items-center justify-between">
            <button
              onClick={onSkip}
              className="text-xs text-zinc-400 dark:text-zinc-500 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
            >
              Passer
            </button>
            <button
              onClick={onNext}
              className="px-5 py-2 bg-blue-500 hover:bg-blue-400 text-white text-sm font-medium rounded-lg transition-colors"
            >
              {isLastStep ? "C'est parti !" : "Suivant"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
