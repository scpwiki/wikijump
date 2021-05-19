<div class="new-w-users-box" id="new-w-users-box">
	{if isset($users)}
		<ul style="list-style: none; margin-left:0; padding-left:1em;">
			{foreach from=$users item=user}
				<li>
					{printuser user=$user image=true} <span class="odate" style="color: #777">{$user->created_at->timestamp}|%O {t}ago{/t}</span>
				</li>
			{/foreach}
		</ul>
	{/if}
</div>
