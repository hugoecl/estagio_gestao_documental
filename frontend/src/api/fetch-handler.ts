import { AlertPosition, AlertType, showAlert } from "@components/alert/alert";
import { toggleElements } from "@stores/loading-stores";

export async function handleFetch(
  url: string | URL,
  options: RequestInit
): Promise<Response> {
  try {
    const response = await fetch(url, options);
    if (
      response.status === 401 &&
      window.location.pathname !== "/iniciar-sessao/" &&
      window.location.pathname !== "/registo/"
    ) {
      window.location.pathname = "/iniciar-sessao/";
    }
    return response;
  } catch (error) {
    toggleElements();

    showAlert(
      "Erro ao comunicar com o servidor",
      AlertType.ERROR,
      AlertPosition.TOP
    );
    throw error;
  }
}
