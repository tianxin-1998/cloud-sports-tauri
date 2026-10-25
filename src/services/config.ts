import { invoke } from "./client";
import type { AppConfig, SchoolList } from "../types";

export const getConfig = () => invoke<AppConfig>("get_config");

export const saveConfig = (config: AppConfig) => invoke<void>("save_config", { config });

export const getSchools = () => invoke<SchoolList>("get_schools");
