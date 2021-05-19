<h1>Page abuse reports</h1>

{if isset($reps)}
	<table class="form alignleft">
		<tr>
			<th>
				Page address
			</th>
			<th>
				Number of flags
			</th>
			<th>
			</th>
		</tr>
		{foreach from=$reps item=rep}
			<tr>
				<td>
					<a href="{$rep.path}" target="_blank">{$rep.path}</a>
				</td>
				<td>
					{$rep.rank}
				</td>
				<td>
					| <a href="javascript:;" onclick="Wikijump.modules.ManageSitePageAbuseModule.listeners.clear(event, '{$rep.path}')">clear flags</a>
				</td>
			</tr>
		{/foreach}
	</table>
{else}
	No abuse reports (yet).
{/if}
