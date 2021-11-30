import argparse
import logging
import os
from typing import List
import fitz
from fitz import Point
from fitz import Rect
from itertools import zip_longest


# https://stackoverflow.com/a/434411/104527
def grouper(iterable, n, fillvalue=None):
    args = [iter(iterable)] * n
    return zip_longest(*args, fillvalue=fillvalue)


class CardWriter:
    def __init__(self, cards_path: str, output: str, side_size: int):
        self.output = output
        self.width, self.height = fitz.paper_size('letter')
        self.side_size = side_size
        self.horizontal_padding = self.width * (1 / 17)
        self.vertical_padding = self.height * (1 / 44)
        self.card_width = (self.width - (self.horizontal_padding * 2)) / side_size
        self.card_height = (self.height - (self.vertical_padding * 2)) / side_size
        self.cards_path = cards_path

    def __draw_guides(
        self,
        shape,
        x0: int,
        y0: float,
        x1: float,
        y1: float,
    ):
        # This all could've been done more elegantly I am certain.
        logging.debug(f'points ({x0}, {y0})\t({x1}, {y1})')
        size = 20
        shape.draw_line(Point(x0, y0 - size), Point(x0, y0 + size))
        shape.draw_line(Point(x0 - size, y0), Point(x0 + size, y0))
        shape.draw_line(Point(x1, y1 - size), Point(x1, y1 + size))
        shape.draw_line(Point(x1 - size, y1), Point(x1 + size, y1))
        shape.draw_line(Point(x0, y1 - size), Point(x0, y1 + size))
        shape.draw_line(Point(x0 - size, y1), Point(x0 + size, y1))
        shape.draw_line(Point(x1, y0 - size), Point(x1, y0 + size))
        shape.draw_line(Point(x1 - size, y0), Point(x1 + size, y0))
        # lol at these hard coded values
        if y0 < 20:
            shape.draw_line(Point(x0, 0), Point(x0, y0))
            shape.draw_line(Point(x1, 0), Point(x1, y0))
        if y0 > 500:
            shape.draw_line(Point(x0, self.height), Point(x0, y1))
            shape.draw_line(Point(x1, self.height), Point(x1, y1))
        if x0 < 40:
            shape.draw_line(Point(0, y0), Point(x0, y0))
            shape.draw_line(Point(0, y1), Point(x0, y1))
        if x0 > 390:
            shape.draw_line(Point(x1, y0), Point(self.width, y0))
            shape.draw_line(Point(x1, y1), Point(self.width, y1))
        shape.finish()
        shape.commit()

    def __group_images(self, images: List[str]):
        groupedRows = grouper(images, self.side_size)
        groupedPages = grouper(groupedRows, self.side_size)
        return groupedPages

    def __images_from_path(self, images_path):
        """
        looks for png, jpg, jpeg, extensions and ignores other files in the given path
        """
        files = os.listdir(images_path)
        images = []
        extensions = ['png', 'jpg', 'jpeg']
        for file in files:
            path = os.path.join(images_path, file)
            if os.path.isfile(path) and any(
                extension in file for extension in extensions
            ):
                images.append(path)
        return self.__group_images(sorted(images))

    def __add_images(self, images: List[List[str]]):
        pdf_page = self.doc.new_page(width=self.width, height=self.height)
        for row_index, row in enumerate(images):
            if row is not None:
                for image_index, image in enumerate(row):
                    if image is not None:
                        x0 = image_index * self.card_width + self.horizontal_padding
                        x1 = x0 + self.card_width
                        y0 = row_index * self.card_height + self.vertical_padding
                        y1 = y0 + self.card_height
                        rect = Rect(x0, y0, x1, y1)
                        img = fitz.open(image)
                        pdfbytes = img.convert_to_pdf()
                        img.close()
                        imgPDF = fitz.open('pdf', pdfbytes)
                        pdf_page.show_pdf_page(rect, imgPDF, 0)
                        shape = pdf_page.new_shape()
                        # Could tell draw guides where we are on the screen
                        # instead of having the magic numbers in that function
                        # determine where to draw the guides from the edges of
                        # sheet. row_index & image_index should be enough here.
                        self.__draw_guides(shape, x0, y0, x1, y1)

    def __align_back_cards(self, back_pages):
        cards = []
        for page in back_pages:
            if page is None:
                cards.append(page)
                continue
            page_cards = []
            cards.append(page_cards)
            for row in page:
                if row is None:
                    page_cards.append(row)
                    continue
                page_cards.append(reversed(list(row)))
        return cards


    def create_pdf(self):
        self.doc = fitz.open()
        front_cards = self.__images_from_path(self.cards_path + '/front')
        back_cards = self.__align_back_cards(self.__images_from_path(self.cards_path + '/back'))
        for image_page in front_cards:
            self.__add_images(image_page)
        self.doc.save(self.output)


def main():
    parser = argparse.ArgumentParser(
        description='Turn directories of images into printable pdfs of card sheets. Cards should fit the aspect ratio '
                    'of 2.5x3.5 '
    )
    parser.add_argument('-c', '--cards-path', type=str,
                        help='Path to the folder containing the card images.')
    parser.add_argument('-o', '--output', type=str, default='cards.pdf',
                        help='Path and filename for the output pdf')
    parser.add_argument('-s', '--sides', type=int, default=3,
                        help='The number of sides in the grid (ex: 3 would produce a 3x3 grid of cards).')
    parser.add_argument(
        '-v', '--verbose', action='store_true', help='Log actions taken at each step.'
    )
    args = parser.parse_args()

    if args.verbose:
        logging.basicConfig(format='%(levelname)s: %(message)s', level=logging.INFO)
    CardWriter(cards_path=args.cards_path, output=args.output, side_size=args.sides).create_pdf()


# i'm not sure how to tell vscode to run __main__.py lmao
if __name__ == '__main__':
    main()
