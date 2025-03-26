import { handleFetch } from "@api/fetch-handler";
import API_BASE_URL from "@api/base-url";

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
