<h2>{t}Your "back" contacts{/t}</h2>

{if isset($contacts)}
	<table class="contact-list-table">
		{foreach from=$contacts item=contact}
			{assign var=user value=$contact->getTargetUser()}
			<tr>
				<td>
					{printuser user=$user image=true}
				</td>
				<td style="padding-left: 5em">
					{$user->username|escape}
				</td>
				</td>
			</tr>
		{/foreach}
	</table>
{else}
	{t}Sorry, no contacts.{/t}
{/if}

