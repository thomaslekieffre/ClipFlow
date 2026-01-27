interface Props {
  size?: number;
  className?: string;
}

export function Logo({ size = 28, className = "" }: Props) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      {/* Rounded rectangle - film frame */}
      <rect x="4" y="4" width="56" height="56" rx="12" fill="url(#grad)" />
      {/* Film perforations left */}
      <rect x="10" y="12" width="5" height="6" rx="1.5" fill="white" opacity="0.3" />
      <rect x="10" y="24" width="5" height="6" rx="1.5" fill="white" opacity="0.3" />
      <rect x="10" y="36" width="5" height="6" rx="1.5" fill="white" opacity="0.3" />
      <rect x="10" y="46" width="5" height="6" rx="1.5" fill="white" opacity="0.3" />
      {/* Play triangle */}
      <path d="M28 20L48 32L28 44V20Z" fill="white" />
      {/* Gradient definition */}
      <defs>
        <linearGradient id="grad" x1="4" y1="4" x2="60" y2="60" gradientUnits="userSpaceOnUse">
          <stop stopColor="#3b82f6" />
          <stop offset="1" stopColor="#8b5cf6" />
        </linearGradient>
      </defs>
    </svg>
  );
}
