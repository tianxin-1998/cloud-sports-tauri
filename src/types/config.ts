export interface AppConfig {
  fast_run: boolean;
  auto_login: boolean;
  default_school_code: string;
  theme: string;
  amap_key: string;
  amap_security: string;
  pace_min: number;
  pace_max: number;
  cadence_min: number;
  cadence_max: number;
}

export const DEFAULT_APP_CONFIG: AppConfig = {
  fast_run: false,
  auto_login: false,
  default_school_code: "",
  theme: "light",
  amap_key: "",
  amap_security: "",
  pace_min: 5.5,
  pace_max: 6.7,
  cadence_min: 164,
  cadence_max: 176,
};
