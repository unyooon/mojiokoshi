export interface Speaker {
  id: string;
  label: string;
  color: SpeakerColor;
  isSelf: boolean;
}

export type SpeakerColor =
  | "blue"
  | "green"
  | "purple"
  | "orange"
  | "pink"
  | "cyan"
  | "red"
  | "yellow";

export const SPEAKER_COLORS: SpeakerColor[] = [
  "blue",
  "green",
  "purple",
  "orange",
  "pink",
  "cyan",
  "red",
  "yellow",
];

export const SPEAKER_COLOR_CLASSES: Record<SpeakerColor, string> = {
  blue: "bg-blue-500",
  green: "bg-green-500",
  purple: "bg-purple-500",
  orange: "bg-orange-500",
  pink: "bg-pink-500",
  cyan: "bg-cyan-500",
  red: "bg-red-500",
  yellow: "bg-yellow-500",
};
