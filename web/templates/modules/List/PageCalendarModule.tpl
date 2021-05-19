<div class="page-calendar-box">
	{if isset($postCount)}
		<ul>
		{foreach from=$postCount key=yearName item=year}
			<li {if $year.selected}class="selected"{/if}>
				<a href="{$startUrlBase}{$yearName}">{$yearName} ({$year.count})</a>
				<ul>
					{foreach from=$year.months key=monthName item=month}
						<li {if $month.selected}class="selected"{/if}>
							<a href="{$startUrlBase}{$yearName}.{$monthName}">{$month.name|escape} ({$month.count})</a>
						</li>
					{/foreach}
				</ul>
			</li>
		{/foreach}
		</ul>
	{else}
		No pages found.
	{/if}
</div>
