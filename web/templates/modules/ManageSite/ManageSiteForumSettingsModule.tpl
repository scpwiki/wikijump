<h1>Forum settings</h1>


{if isset($forumSettings)}

	<h2>Default nesting</h2>

	<p>
		Discussions can have different structures: nested (threaded) of flat (linear).
	</p>
	<p>
		In the nested mode
		it is possible (up to the specified nesting level) to answer to any particular post in the
		discussion, create digressions etc. Replies have larger identation.
	</p>
	<p>
		In the linear mode each new post is appended at the end of the topic which does not allow creating
		independent paths.
	</p>
	<p>
		WARNING: please note that changing nesting for an active forum can confuse users.
	</p>

	<div style="text-align: center">
		default nesting level:
			<select  id="max-nest-level">
				<option value="0" {if $forumSettings->getMaxNestLevel() == 0}selected="selected"{/if}>0 (flat/linear)</option>
				<option value="1" {if $forumSettings->getMaxNestLevel() == 1}selected="selected"{/if}>1</option>
				<option value="2" {if $forumSettings->getMaxNestLevel() == 2}selected="selected"{/if}>2</option>
				<option value="3" {if $forumSettings->getMaxNestLevel() == 3}selected="selected"{/if}>3</option>
				<option value="4" {if $forumSettings->getMaxNestLevel() == 4}selected="selected"{/if}>4</option>
				<option value="5" {if $forumSettings->getMaxNestLevel() == 5}selected="selected"{/if}>5</option>
				<option value="6" {if $forumSettings->getMaxNestLevel() == 6}selected="selected"{/if}>6</option>
				<option value="7" {if $forumSettings->getMaxNestLevel() == 7}selected="selected"{/if}>7</option>
				<option value="8" {if $forumSettings->getMaxNestLevel() == 8}selected="selected"{/if}>8</option>
				<option value="9" {if $forumSettings->getMaxNestLevel() == 9}selected="selected"{/if}>9</option>
				<option value="10" {if $forumSettings->getMaxNestLevel() == 10}selected="selected"{/if}>10</option>
			</select>
			<input class="button" type="button" value="save nesting" onclick="Wikijump.modules.ManageSiteForumSettingsModule.listeners.saveNesting()"/>

	</div>

	{*<h2>Votes &amp; karma</h2>

	<p>
		This is not implemented yet, but will allow users to "vote" on posts.
		The posts wich will get negative rank will be by default folded or hidden in the thread view.
	</p>*}

	<h2>Forum special pages</h2>

	<p>
		Forum functions use a few special pages. If you have just activated the forum
		it is very likely that your site does not contain links to these pages.
	</p>
	<p>
		The special forum pages are:
	</p>
	<code><pre>
forum:start
forum:category
forum:thread
forum:new-thread
forum:recent-posts</pre></code>

	<p>
		Make sure you have a link somewhere to <a href="/forum:start"><tt>forum:start</tt></a> and possibly to
		<a href="/forum:recent-posts"><tt>forum:recent-posts</tt></a> but nothing else.<br/>
		Good places are certainly <a href="/nav:top"><tt>nav:top</tt></a> or
		<a href="/nav:side"><tt>nav:side</tt></a>. An example code to copy&amp;paste is given below:
	</p>

	<div class="code hl-main">
	<pre>
* [[[forum:start | Forum]]]
* [[[forum:recent-posts | Recent posts]]]</pre>
	</div>
	<p>
		<!-- TODO: De-Wikijump.com-ize - change -->
		For a complete guide on starting a forum look at this <a href="http://community.wikijump.com/howto:forum-step-by-step" target="_blank">Step-by-step Howto</a>.
	</p>
{else}

	<h2>Activate forum</h2>

	<p>
		In order to use forum (and per-page discussions) you must initialize the forum structures in
		your Wikijump site. When you decide to use the forum additional pages will be created
		for you automatically. After that you will be able to create forum categories
		structure, set permissions etc.
	</p>
	<div class="buttons">
		<input type="button" value="activate forum now" onclick="Wikijump.modules.ManageSiteForumSettingsModule.listeners.activateForum(event)"/>
	</div>

	<p>
		<!-- TODO: De-Wikijump.com-ize - change -->
		For a complete guide on starting a forum look at this <a href="http://community.wikijump.com/howto:forum-step-by-step" target="_blank">Step-by-step Howto</a>.
	</p>

{/if}
