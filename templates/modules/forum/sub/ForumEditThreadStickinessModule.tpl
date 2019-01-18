<h2>{t}Edit thread stickness{/t}</h2>
<p>
	{t}If a thread is "sticky" - it remains on top of the threads list in the 
	forum category view. This feature is recommended for threads of
	exceptional importance.{/t}
</p>

{*
<p style="text-align: center">
	<input type="checkbox" id="thread-sticky-checkbox" {if $thread->getSticky() == true}checked="checked"{/if}/> has sticky state
	<input type="button" value="save" onclick="WIKIDOT.modules.ForumEditThreadStickinessModule.listeners.save(event)"/>
	<input type="button" value="cancel" onclick="$('thread-action-area').style.display='none'"/>
</p>
*}

<form>
	<table class="form">
		<tr>
			<td>
				{t}Has sticky state{/t}:
			</td>
			<td>
				<input class="checkbox" type="checkbox" id="thread-sticky-checkbox" {if $thread->getSticky() == true}checked="checked"{/if}/>
			</td>
		</tr>
	</table>
	<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="$('thread-action-area').style.display='none'"/>
			<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ForumEditThreadStickinessModule.listeners.save(event)"/>
	</div>
</form>