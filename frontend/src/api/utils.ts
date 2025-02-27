import API_BASE_URL from "@api/base-url";

import { showAlert, AlertPosition, AlertType } from "@components/Alert/Alert";

async function handleFetch(
  url: string | URL,
  options: RequestInit
): Promise<Response> {
  try {
    const response = await fetch(url, options);
    return response;
  } catch (error) {
    showAlert(
      "Erro ao comunicar com o servidor",
      AlertType.ERROR,
      AlertPosition.TOP
    );
    throw error;
  }
}

export async function registerUser(
  username: string,
  email: string,
  password: string
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/users/register`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ username, email, password }),
  });
  return response.ok;
}

export async function loginUser(
  email: string,
  password: string
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/users/login`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ email, password }),
  });

  return response.ok;
}

export async function logoutUser(): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/users/logout`, {
    method: "POST",
    credentials: "include",
  });
  return response.ok;
}
