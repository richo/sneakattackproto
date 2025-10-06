default: update_rallies html/timecomp.html


update_rallies:
	curl https://sneakattackrally.com/ARACombinerThing/data/nonARA/allRallies.json > nonARArallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/2024rallies.json > 2024rallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/2025rallies.json > 2025rallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/uidsSmall.json > uidsSmall.json

html/timecomp.html: html/timecomp.html.erb html/generate-form.rb 2024rallies.json 2025rallies.json
	cd html && ruby generate-form.rb

copy: html/timecomp.html
	cp html/timecomp.html ~/src/recce.tools/


.PHONY: default update_rallies copy
