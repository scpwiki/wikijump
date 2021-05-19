<h1>IP Address Blocks</h1>

<p>
	To prevent {$SERVICE_NAME} sites (including this one of course) abuse there is a possibility to
	block IP addresses from which sucpicious users might alter contents of the site.
</p>

<h2>This site's IP block list</h2>
{if isset($blocks)}
	<ul style="list-style: none">
		{foreach from=$blocks item=block}
			<li style="margin: 0.2em 0">
				<span style="font-size: 110%; font-weight: bold">{$block->getIp()|escape}</span>
				<br/>
				<div style="position: absolute; margin-left: 30em">
					<a href="javascript:;" onclick="Wikijump.modules.ManageSiteIpBlocksModule.listeners.deleteBlock(event, {$block->getBlockId()}, '{$block->getIp()|escape}')">delete block</a>
				</div>
				blocked on: <span class="odate">{$block->getDateBlocked()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
				{if $block->getReason() && $block->getReason() != ''}
					<br/>reason: {$block->getReason()|escape}
				{/if}
			</li>
		{/foreach}
	</ul>
{else}
	No IP blocks for this site.
{/if}

<div id="show-add-block-button">
	<a href="javascript:;" onclick="Wikijump.modules.ManageSiteIpBlocksModule.listeners.showAddForm(event)">+add new IP block</a>
</div>

<div id="add-block-div" style="display: none">
	<h2>Add new IP block(s)</h2>
	<div class="help-block">
		Please add IP addresses <u>one entry per line</u> using one of the formats listed below:
		<ul>
			<li>
				123.45.67.89 - blocks a single host
			</li>
			<li>
				123.45.67.0/24 or 123.45.67.* - blocks a range of IP addresses from 123.45.67.1.1 to 123.45.67.255
			</li>
			<li>
				123.45.0.0/16 or 123.45.*.* - blocks a range of addresses from 123.45.0.1 to 123.45.255.255
			</li>
		</ul>
	<p>
		Please do not submit IP addresses from private ranges.
	</p>
	<p>
		Please also keep in mind that <b>blocking an IP address might be very harmful</b>
		for several reasons and might affect innocent users while not being effective against some vandals.
		Just use with caution.
	</p>
	</div>
	<div id="ip-errors" class="error-block" style="display: none"></div>
	<table class="form">
		<tr>
			<td>
				IP address(es):
			</td>
			<td>
				<textarea cols="40" rows="6" id="block-ips"></textarea>
			</td>
		</tr>
		<tr>
			<td>
				Reason:
			</td>
			<td>
				<textarea id="block-reason" cols="40" rows="6"></textarea>
				<div class="sub">
					<span id="reason-char-left"></span> characters left.
				</div>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="cancel" onclick="Wikijump.modules.ManageSiteIpBlocksModule.listeners.cancelAdd(event)"/>
		<input type="button" value="block" onclick="Wikijump.modules.ManageSiteIpBlocksModule.listeners.blockIp(event)"/>
	</div>
	<p>
		Note: The added block will affect both www proxies and client IP addresses.
	</p>
</div>
