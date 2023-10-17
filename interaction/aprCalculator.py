from rich import print
from rich.panel import Panel
from rich.prompt import Prompt

USDC_TO_SHARES = 1000000  # Conversion of 1 USDC to shares

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

def calculate_rewards(duration_in_days: float, apr: float, amount_in_usdc: float) -> float:
    number_of_shares = amount_in_usdc * USDC_TO_SHARES
    rewards_per_block = apr_to_rewards_per_block(apr)
    blocks_per_day = (60 * 60 * 24) / 6
    daily_reward = rewards_per_block * blocks_per_day * number_of_shares
    total_reward_for_duration = daily_reward * duration_in_days
    return total_reward_for_duration

if __name__ == "__main__":
    print(Panel("[bold green]Conversion Tool[/bold green]"))

    options = {
        "1": "Convert APR to rewardsPerSharePerBlock",
        "2": "Convert rewardsPerSharePerBlock to APR",
        "3": "Calculate rewards over a given duration and APR for a specified USDC amount"
    }

    for key, value in options.items():
        print(f"[bold cyan]{key}:[/bold cyan] {value}")

    choice = Prompt.ask("Enter your choice", choices=["1", "2", "3"])

    if choice == "1":
        apr_value = float(Prompt.ask("Enter the APR (e.g., for 3%, input 0.03)"))
        print(f"rewardsPerSharePerBlock: [bold yellow]{apr_to_rewards_per_block(apr_value):.12f}[/bold yellow]")

    elif choice == "2":
        rewards_value = float(Prompt.ask("Enter the rewardsPerSharePerBlock"))
        print(f"APR: [bold yellow]{rewards_per_block_to_apr(rewards_value) * 100:.2f}%[/bold yellow]")
    elif choice == "3":
        apr_value = float(Prompt.ask("Enter the APR (e.g., for 3%, input 0.03)"))
        duration = float(Prompt.ask("Enter the duration in days"))
        usdc_amount = float(Prompt.ask("Enter the amount in USDC"))
        total_rewards_in_shares = calculate_rewards(duration, apr_value, usdc_amount)
        total_rewards_in_usdc = total_rewards_in_shares / USDC_TO_SHARES
        print(f"Total reward for [bold yellow]{usdc_amount}[/bold yellow] USDC (equivalent to [bold yellow]{usdc_amount * USDC_TO_SHARES}[/bold yellow] shares) over [bold yellow]{duration}[/bold yellow] days:")
        print(f"Shares: [bold yellow]{total_rewards_in_shares:.15f}[/bold yellow]")
        print(f"USDC: [bold yellow]{total_rewards_in_usdc:.6f}[/bold yellow]")

    else:
        print("[bold red]Invalid choice![/bold red]")
