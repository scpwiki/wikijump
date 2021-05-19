<div class="most-active-forums-box">
	<div class="when">
		{if $range=='24h'}
			24 {t}hours{/t}
		{else}
			<a href="javascript:;" onclick="Wikijump.modules.MostActiveForumsModule.listeners.changeTime(event, '24h')"> 24 {t}hours{/t}</a>
		{/if}
		|
		{if $range=='7days'}
			7 {t}days{/t}
		{else}
			<a href="javascript:;" onclick="Wikijump.modules.MostActiveForumsModule.listeners.changeTime(event, '7d')"> 7 {t}days{/t}</a>
		{/if}
		|
		{if $range=='month'}
			{t}month{/t}
		{else}
			<a href="javascript:;" onclick="Wikijump.modules.MostActiveForumsModule.listeners.changeTime(event, 'month')"> {t}month{/t}</a>
		{/if}
	</div>

	{if isset($res)}
		<table class="item-list">
			<tr>
				<td>&nbsp;</td>
				<td>{t}posts{/t}</td>
			</tr>
			{foreach from=$res item=r}
				<tr>
					<td>
						<a href="{$HTTP_SCHEMA}://{$r.site->getDomain()}">{$r.site->getName()|escape}</a>
					</td>
					<td style="text-align: right">
						{$r.number_posts}
					</td>
				</tr>
			{/foreach}
		</table>
	{else}
		<p>
			{t}Sorry, no activity in this range.{/t}
		</p>
	{/if}

</div>
