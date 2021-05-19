{if isset($users)}
	{foreach from=$users item=user}
		<div id="found-user-{$user->id}">
			{printuser user=$user image="yes"}
		</div>
	{/foreach}
{else}
	{t}Sorry, no users found.{/t}
{/if}


