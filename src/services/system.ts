import { invoke } from "./client";
import type { UpstreamJson } from "../types";

export const ping = () => invoke<string>("ping");

export const status = () => invoke<{ server_time: number }>("status");

export const quitApp = () => invoke<void>("quit_app");

export const openExternal = (url: string) => invoke<void>("open_external", { url });

export const getTask = (taskId: string) => invoke<UpstreamJson>("get_task", { taskId });
