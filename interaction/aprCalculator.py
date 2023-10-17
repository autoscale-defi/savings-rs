from rich import print
from rich.panel import Panel
from rich.prompt import Prompt

def apr_to_rewards_per_block(apr: float) -> float:
    blocks_per_day = (60 * 60 * 24) / 6
    daily_reward_per_share = apr / 365
    rewards_per_share_per_block = daily_reward_per_share / blocks_per_day
    return rewards_per_share_per_block

def rewards_per_block_to_apr(rewards_per_block: float) -> float:
    blocks_per_day = (60 * 60 * 24) / 6
    daily_reward_per_share = rewards_per_block * blocks_per_day
    apr = daily_reward_per_share * 365
    return apr

def calculate_rewards(duration_in_days: float, apr: float, number_of_shares: float) -> float:
    rewards_per_block = apr_to_rewards_per_block(apr)
    blocks_per_day = (60 * 60 * 24) / 6
    daily_reward = rewards_per_block * blocks_per_day * number_of_shares
    total_reward_for_duration = daily_reward * duration_in_days
    return total_reward_for_duration

if __name__ == "__main__":
    print(Panel("[bold green]Outil de Conversion[/bold green]"))

    options = {
        "1": "Convertir APR en rewardsPerSharePerBlock",
        "2": "Convertir rewardsPerSharePerBlock en APR",
        "3": "Calculer les rewards sur une durée donnée et un APR donné avec un nombre de shares spécifié"
    }

    for key, value in options.items():
        print(f"[bold cyan]{key}:[/bold cyan] {value}")

    choice = Prompt.ask("Entrez votre choix", choices=["1", "2", "3"])

    if choice == "1":
        apr_value = float(Prompt.ask("Entrez l'APR (par exemple, pour 3%, entrez 0.03)"))
        print(f"rewardsPerSharePerBlock: [bold yellow]{apr_to_rewards_per_block(apr_value):.15f}[/bold yellow]")

    elif choice == "2":
        rewards_value = float(Prompt.ask("Entrez le rewardsPerSharePerBlock"))
        print(f"APR: [bold yellow]{rewards_per_block_to_apr(rewards_value) * 100:.2f}%[/bold yellow]")

    elif choice == "3":
        apr_value = float(Prompt.ask("Entrez l'APR (par exemple, pour 3%, entrez 0.03)"))
        duration = float(Prompt.ask("Entrez la durée en jours"))
        shares = float(Prompt.ask("Entrez le nombre de shares"))
        print(f"Récompense totale pour [bold yellow]{shares}[/bold yellow] shares pendant [bold yellow]{duration}[/bold yellow] jours: [bold yellow]{calculate_rewards(duration, apr_value, shares):.15f}[/bold yellow]")

    else:
        print("[bold red]Choix invalide![/bold red]")
