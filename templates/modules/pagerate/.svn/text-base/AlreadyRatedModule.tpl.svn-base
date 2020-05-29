<div class="owindow">
	<div class="title">
		{t}Already rated{/t}
	</div>
	<div class="content">
		<h1>{t}Trying to re-rate this content?{/t}</h1>
		<p>
			{t}It seems you have already rated this content and your current rating is:{/t}
			<b>{if $rate != 0}{$rate|string_format:"%+d"}{else}{$rate}{/if}</b>.
		</p>
		
		{if $rate == $points}
			<p>
				{t}You can not "re-rate" it with the same value.
				What you can do is to: change your mind and your vote or cancel your vote.{/t}
			</p>
		{else}	
			<p>
				{t}Do you really want to change your mind and rate this with{/t}
				<b>{if $points != 0}{$points|string_format:"%+d"}{else}{$points}{/if}</b> points?
			</p>
		{/if}
	</div>
	<div class="button-bar">
		{if $rate == $points}
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll();">{t}close window{/t}</a>
		{else}
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll();">{t}cancel{/t}</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, {$points}, true)">rerate!</a>
		{/if}
	</div>
</div>