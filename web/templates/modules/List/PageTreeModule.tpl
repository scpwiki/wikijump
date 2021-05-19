{defmacro name="printChildren"}
	{assign var=chs value=$children[$rootId]}
	{if isset($chs)}
		<ul>
			{foreach from=$chs item=ch}
				<li>
					<a href="/{$ch->getUnixName()}">{$ch->getTitleOrUnixName()|escape}</a>
					{if $ch->getTemp('circular')}
						Warning: circular parenthood.
					{else}
						{macro name="printChildren" rootId=$ch->getPageId() children=$children}
					{/if}
				</li>
			{/foreach}
		</ul>
	{/if}
{/defmacro}

{assign var=rootId value=$root->getPageId()}

{if isset($showRoot)}
	<ul>
		<li>
			<a href="/{$root->getUnixName()}">{$root->getTitleOrUnixName()|escape}</a>
			{macro name="printChildren" rootId=$rootId children=$children}
		</li>
	</ul>
{else}
	{macro name="printChildren" rootId=$rootId children=$children}
{/if}



