import os, sys, fitz

imgdir = sys.argv[1]

doc = fitz.open()

width, height = fitz.paper_size("letter")
imglist = os.listdir(imgdir)
imglist = imglist + imglist + imglist
imglist.sort()
imgcount = len(imglist)

page = doc.new_page(width=width, height=height)
# try to use this template as the base page, dunno why it isn't working
# pdf_path = os.path.join(imgdir, 'template.pdf')
# doc = fitz.open()

# 2.5/3.5 = 0.7142857143
# 1572 wide, 2038 tall
horizontal_padding = width * (1/17)
vertical_padding = height * (1/44)
card_width = (width - (horizontal_padding * 2)) / 3
card_height = (height - (vertical_padding * 2)) / 3
count = 0
row = 0
column = 0
for i, f in enumerate(imglist):
    path = os.path.join(imgdir, f)
    if not os.path.isfile(path) or 'DS_Store' in path or 'pdf' in path:
        continue
    print(f'index: {count}, row: {row}, column: {column}')
    x0 = column * card_width + horizontal_padding
    x1 = x0 + card_width
    y0 = row * card_height  + vertical_padding
    y1 = y0 + card_height

    img = fitz.open(path)
    rect = (x0, y0, x1, y1)
    print(f'rect: {rect}')
    pdfbytes = img.convert_to_pdf()
    img.close()
    imgPDF = fitz.open("pdf", pdfbytes)
    page.show_pdf_page(rect, imgPDF, 0)
    count = count + 1
    if count == 3 or count == 6:
        row = row + 1
    column = count % 3

    # for a new page call
    #page = doc.new_page(width = width, height = height)

doc.save("all-my-pics-embedded.pdf")