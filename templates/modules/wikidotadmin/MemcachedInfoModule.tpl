<h1>Memcached Server(s) Info</h1>

{foreach from=$raw key=key item=server}
	<h2>{$key}</h2>
	{foreach from=$server key=prop item=val}
		<strong>{$prop}</strong>: {$val}<br/>
	{/foreach}
{/foreach}