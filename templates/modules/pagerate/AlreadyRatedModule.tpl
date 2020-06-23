<div class="owindow">
	<div class="title">
		{t}Already rated{/t}
	</div>
	<div class="content">
		<h1>			{t}You have already rated this page {/t}
			<b>{if $rate != 0}{$rate|round|string_format:"%+d"}{else}{$rate|round}{/if}</b>.
		</h1>
		
		{if $rate == $points}
			<p>
				{t}You can cancel your vote by clicking on the x button.{/t}
			</p>
		{else}	
			<p>
				{t}Would you like to change your vote to {/t}
				<b>{if $points != 0}{$points|string_format:"%+d"}{else}{$points}{/if}</b>?
			</p>
		{/if}
	</div>
	<div class="button-bar">
		{if $rate == $points}
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll();">{t}close window{/t}</a>
		{else}
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll();">{t}Cancel{/t}</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, {$points}, true)">Rerate</a>
		{/if}
	</div>
</div>